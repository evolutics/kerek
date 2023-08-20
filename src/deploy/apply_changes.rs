use super::model;
use crate::command;
use anyhow::Context;
use std::collections;
use std::process;
use std::thread;
use std::time;

pub fn go(in_: In) -> anyhow::Result<()> {
    let mut state = new_rolling_state(in_.actual_containers);

    for change in in_.changes {
        let summary = summarize_change(change);
        eprintln!("Going to {summary}.");
        apply_change(change, in_.docker_host, &mut state)
            .with_context(|| format!("Unable to {summary}"))?;
    }

    Ok(())
}

pub struct In<'a> {
    pub actual_containers: &'a model::ActualContainers,
    pub changes: &'a [model::ServiceContainerChange],
    pub desired_state: &'a model::DesiredState,
    pub docker_host: &'a str,
}

struct RollingState<'a> {
    service_container_count: collections::BTreeMap<&'a str, u16>,
}

fn new_rolling_state(actual_containers: &model::ActualContainers) -> RollingState {
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

fn summarize_change(change: &model::ServiceContainerChange) -> String {
    // TODO: Display shortened container ID for brevity.
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
    &hash[..8]
}

fn summarize_service(service_name: &str, service_config_hash: &str) -> String {
    let service_config_hash = summarize_hash(service_config_hash);
    format!("service {service_name:?} with config hash {service_config_hash}")
}

fn apply_change<'a>(
    change: &'a model::ServiceContainerChange,
    docker_host: &str,
    state: &mut RollingState<'a>,
) -> anyhow::Result<()> {
    match change {
        model::ServiceContainerChange::Add { service_name, .. } => {
            add_container(service_name, docker_host, state)
        }

        model::ServiceContainerChange::Keep { .. } => Ok(()),

        model::ServiceContainerChange::Remove {
            container_id,
            service_name,
            ..
        } => remove_container(service_name, container_id, docker_host, state),
    }
}

fn add_container<'a>(
    service_name: &'a str,
    docker_host: &str,
    state: &mut RollingState<'a>,
) -> anyhow::Result<()> {
    let container_count = state
        .service_container_count
        .entry(service_name)
        .and_modify(|count| *count += 1)
        .or_insert(1);

    command::status_ok(process::Command::new("docker").args([
        "--host",
        docker_host,
        "compose",
        "up",
        "--detach",
        "--no-recreate",
        "--scale",
        &format!("{service_name}={container_count}"),
        "--",
        service_name,
    ]))?;

    // TODO: Wait until healthy, e.g. using "--wait".
    thread::sleep(time::Duration::from_secs(2));

    Ok(())
}

fn remove_container<'a>(
    service_name: &'a str,
    container_id: &str,
    docker_host: &str,
    state: &mut RollingState<'a>,
) -> anyhow::Result<()> {
    command::status_ok(process::Command::new("docker").args([
        "--host",
        docker_host,
        "stop",
        "--",
        container_id,
    ]))?;

    command::status_ok(process::Command::new("docker").args([
        "--host",
        docker_host,
        "rm",
        "--",
        container_id,
    ]))?;

    state
        .service_container_count
        .entry(service_name)
        .and_modify(|count| *count -= 1);

    Ok(())
}
