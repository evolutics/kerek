use super::model;
use crate::command;
use crate::docker;
use crate::docker_compose;
use crate::log;
use anyhow::Context;
use std::collections;

pub fn go(
    In {
        actual_containers,
        build,
        changes,
        docker_cli,
        docker_compose_cli,
        dry_run,
        no_build,
        no_start,
        pull,
        quiet_pull,
        remove_orphans,
        renew_anon_volumes,
        service_names,
        timeout,
        wait,
        wait_timeout,
    }: In,
) -> anyhow::Result<()> {
    let mut state = new_rolling_state(actual_containers);

    if build {
        build_images(service_names, dry_run, docker_compose_cli)?;
    }

    for change in changes {
        let summary = summarize_change(change);

        if dry_run {
            log::info!("Would {summary}.");
        } else {
            log::info!("Going to {summary}.");
            apply_change(
                change,
                ChangeOptions {
                    no_build,
                    no_start,
                    pull,
                    quiet_pull,
                    remove_orphans,
                    renew_anon_volumes,
                    timeout,
                    wait,
                    wait_timeout,
                },
                docker_cli,
                docker_compose_cli,
                &mut state,
            )
            .with_context(|| format!("Unable to {summary}"))?;
        }
    }

    Ok(())
}

pub struct In<'a> {
    pub actual_containers: &'a model::ActualContainers,
    pub build: bool,
    pub changes: &'a [model::ServiceContainerChange],
    pub docker_cli: &'a docker::Cli<'a>,
    pub docker_compose_cli: &'a docker_compose::Cli<'a>,
    pub dry_run: bool,
    pub no_build: bool,
    pub no_start: bool,
    pub pull: Option<&'a str>,
    pub quiet_pull: bool,
    pub remove_orphans: bool,
    pub renew_anon_volumes: bool,
    pub service_names: &'a [&'a String],
    pub timeout: Option<&'a str>,
    pub wait: bool,
    pub wait_timeout: Option<&'a str>,
}

struct RollingState<'a> {
    service_container_count: collections::BTreeMap<&'a str, u16>,
}

struct ChangeOptions<'a> {
    no_build: bool,
    no_start: bool,
    pull: Option<&'a str>,
    quiet_pull: bool,
    remove_orphans: bool,
    renew_anon_volumes: bool,
    timeout: Option<&'a str>,
    wait: bool,
    wait_timeout: Option<&'a str>,
}

fn new_rolling_state(actual_containers: &model::ActualContainers) -> RollingState<'_> {
    let mut service_container_count = collections::BTreeMap::new();

    for container in actual_containers {
        service_container_count
            .entry(container.service_name.as_ref())
            .and_modify(|count| *count += 1)
            .or_insert(1);
    }

    RollingState {
        service_container_count,
    }
}

fn build_images(
    service_names: &[&String],
    dry_run: bool,
    docker_compose_cli: &docker_compose::Cli,
) -> anyhow::Result<()> {
    log::debug!("Building services.");
    command::status_ok(
        docker_compose_cli
            .command()
            .args(dry_run.then_some("--dry-run").iter())
            .args(["build", "--"])
            .args(service_names),
    )
}

fn summarize_change(change: &model::ServiceContainerChange) -> String {
    match change {
        model::ServiceContainerChange::Add {
            service_config_hash,
            service_name,
        } => {
            let service = summarize_service(service_name, service_config_hash);
            format!("add a container of {service}")
        }
        model::ServiceContainerChange::Keep {
            container_id,
            service_config_hash,
            service_name,
        } => {
            let container = summarize_container(container_id);
            let service = summarize_service(service_name, service_config_hash);
            format!("keep the {container} of {service}")
        }
        model::ServiceContainerChange::Remove {
            container_id,
            service_config_hash,
            service_name,
        } => {
            let container = summarize_container(container_id);
            let service = summarize_service(service_name, service_config_hash);
            format!("remove the {container} of {service}")
        }
    }
}

fn summarize_container(container_id: &str) -> String {
    let container_id = summarize_hash(container_id);
    format!("container {container_id}")
}

fn summarize_hash(hash: &str) -> &str {
    if log::level() <= log::Level::Debug {
        hash
    } else {
        &hash[..8]
    }
}

fn summarize_service(service_name: &str, service_config_hash: &str) -> String {
    let service_config_hash = summarize_hash(service_config_hash);
    format!("service {service_name:?} with config hash {service_config_hash}")
}

fn apply_change<'a>(
    change: &'a model::ServiceContainerChange,
    change_options: ChangeOptions,
    docker_cli: &docker::Cli,
    docker_compose_cli: &docker_compose::Cli,
    state: &mut RollingState<'a>,
) -> anyhow::Result<()> {
    match change {
        model::ServiceContainerChange::Add { service_name, .. } => {
            add_container(service_name, change_options, docker_compose_cli, state)
        }

        model::ServiceContainerChange::Keep { .. } => Ok(()),

        model::ServiceContainerChange::Remove {
            container_id,
            service_name,
            ..
        } => remove_container(service_name, container_id, docker_cli, state),
    }
}

fn add_container<'a>(
    service_name: &'a str,
    ChangeOptions {
        no_build,
        no_start,
        pull,
        quiet_pull,
        remove_orphans,
        renew_anon_volumes,
        timeout,
        wait,
        wait_timeout,
    }: ChangeOptions,
    docker_compose_cli: &docker_compose::Cli,
    state: &mut RollingState<'a>,
) -> anyhow::Result<()> {
    let container_count = state
        .service_container_count
        .entry(service_name)
        .and_modify(|count| *count += 1)
        .or_insert(1);

    log::debug!("Scaling service {service_name:?} to {container_count} instances.");
    command::status_ok(
        docker_compose_cli
            .command()
            .args(["up", "--detach"])
            .args(no_build.then_some("--no-build").iter())
            .args(["--no-deps", "--no-recreate"])
            .args(no_start.then_some("--no-start").iter())
            .args(pull.iter().flat_map(|pull| ["--pull", pull]))
            .args(quiet_pull.then_some("--quiet-pull").iter())
            .args(remove_orphans.then_some("--remove-orphans").iter())
            .args(renew_anon_volumes.then_some("--renew-anon-volumes").iter())
            .args(["--scale", &format!("{service_name}={container_count}")])
            .args(timeout.iter().flat_map(|timeout| ["--timeout", timeout]))
            .args(wait.then_some("--wait").iter())
            .args(
                wait_timeout
                    .iter()
                    .flat_map(|wait_timeout| ["--wait-timeout", wait_timeout]),
            )
            .args(["--", service_name]),
    )
}

fn remove_container<'a>(
    service_name: &'a str,
    container_id: &str,
    docker_cli: &docker::Cli,
    state: &mut RollingState<'a>,
) -> anyhow::Result<()> {
    let container = summarize_container(container_id);

    log::debug!("Stopping {container}.");
    command::status_ok(docker_cli.command().args(["stop", "--", container_id]))?;

    log::debug!("Removing {container}.");
    command::status_ok(docker_cli.command().args(["rm", "--", container_id]))?;

    state
        .service_container_count
        .entry(service_name)
        .and_modify(|count| *count -= 1);

    Ok(())
}
