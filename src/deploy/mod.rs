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
        docker_cli,
        dry_run,
        service_names,
    }: In,
) -> anyhow::Result<()> {
    // TODO: Handle stopped containers.

    let actual_containers = get_actual_state::go(&service_names, &docker_cli)?;
    let desired_services = get_desired_state::go(&service_names, &docker_cli)?;
    let changes = plan_changes::go(&actual_containers, &desired_services);

    apply_changes::go(apply_changes::In {
        actual_containers: &actual_containers,
        build,
        changes: &changes,
        docker_cli: &docker_cli,
        dry_run,
        service_names: &service_names,
    })
}

pub struct In {
    pub build: bool,
    pub docker_cli: docker::Cli,
    pub dry_run: bool,
    pub service_names: collections::BTreeSet<String>,
}
