mod apply_changes;
mod get_actual_state;
mod get_desired_state;
mod model;
mod plan_changes;

use super::docker;
use super::docker_compose;
use std::collections;

pub fn go(
    In {
        build,
        docker_cli,
        docker_compose_cli,
        dry_run,
        force_recreate,
        no_build,
        no_deps,
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
    let desired_services = get_desired_state::go(&service_names, &docker_compose_cli, no_deps)?;
    let service_names = if service_names.is_empty() {
        vec![]
    } else {
        desired_services.keys().collect()
    };
    let actual_containers = get_actual_state::go(&service_names, &docker_cli, &docker_compose_cli)?;
    let changes = plan_changes::go(&actual_containers, &desired_services, force_recreate);

    apply_changes::go(apply_changes::In {
        actual_containers: &actual_containers,
        build,
        changes: &changes,
        docker_cli: &docker_cli,
        docker_compose_cli: &docker_compose_cli,
        dry_run,
        no_build,
        no_start,
        pull: pull.as_deref(),
        quiet_pull,
        remove_orphans,
        renew_anon_volumes,
        service_names: &service_names,
        timeout: timeout.as_deref(),
        wait,
        wait_timeout: wait_timeout.as_deref(),
    })
}

pub struct In<'a> {
    pub build: bool,
    pub docker_cli: docker::Cli<'a>,
    pub docker_compose_cli: docker_compose::Cli<'a>,
    pub dry_run: bool,
    pub force_recreate: bool,
    pub no_build: bool,
    pub no_deps: bool,
    pub no_start: bool,
    pub pull: Option<String>,
    pub quiet_pull: bool,
    pub remove_orphans: bool,
    pub renew_anon_volumes: bool,
    pub service_names: collections::BTreeSet<String>,
    pub timeout: Option<String>,
    pub wait: bool,
    pub wait_timeout: Option<String>,
}
