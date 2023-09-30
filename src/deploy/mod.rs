mod apply_changes;
mod get_actual_state;
mod get_desired_state;
mod model;
mod plan_changes;

use super::docker;
use std::collections;

pub fn go(
    In {
        build,
        detach,
        docker_cli,
        dry_run,
        force_recreate,
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
    let actual_containers = get_actual_state::go(&service_names, &docker_cli)?;
    let desired_services = get_desired_state::go(&service_names, &docker_cli)?;
    let changes = plan_changes::go(&actual_containers, &desired_services, force_recreate);

    apply_changes::go(apply_changes::In {
        actual_containers: &actual_containers,
        build,
        changes: &changes,
        detach,
        docker_cli: &docker_cli,
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

pub struct In {
    pub build: bool,
    pub detach: bool,
    pub docker_cli: docker::Cli,
    pub dry_run: bool,
    pub force_recreate: bool,
    pub no_build: bool,
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
