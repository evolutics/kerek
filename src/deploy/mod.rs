mod apply_changes;
mod get_actual_state;
mod get_desired_state;
mod model;
mod plan_changes;

use super::docker;

pub fn go(in_: In) -> anyhow::Result<()> {
    // TODO: Handle stopped containers.

    let actual_containers = get_actual_state::go(&in_.docker_cli)?;
    let desired_state = get_desired_state::go(&in_.docker_cli)?;
    let changes = plan_changes::go(&actual_containers, &desired_state.services);

    apply_changes::go(apply_changes::In {
        actual_containers: &actual_containers,
        changes: &changes,
        desired_state: &desired_state,
        docker_cli: &in_.docker_cli,
        dry_run: in_.dry_run,
    })
}

pub struct In {
    pub docker_cli: docker::Cli,
    pub dry_run: bool,
}
