mod apply_changes;
mod get_actual_state;
mod get_desired_state;
mod model;
mod plan_changes;

use crate::library::docker_host;

pub fn go(in_: In) -> anyhow::Result<()> {
    // TODO: Handle stopped containers.

    let docker_host = docker_host::get(in_.docker_host)?.url;

    let actual_containers = get_actual_state::go(&docker_host)?;
    let desired_state = get_desired_state::go(&docker_host)?;
    let changes = plan_changes::go(&actual_containers, &desired_state.services);

    apply_changes::go(apply_changes::In {
        actual_containers: &actual_containers,
        changes: &changes,
        desired_state: &desired_state,
        docker_host: &docker_host,
    })
}

pub struct In {
    pub compose_file: String, // TODO: Implement.
    pub docker_host: Option<String>,
    pub project_folder: Option<String>, // TODO: Implement.
    pub project_name: Option<String>,   // TODO: Implement.
}
