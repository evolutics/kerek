use super::model;
use crate::library::command;
use std::collections;
use std::process;

pub fn go(docker_host: &str) -> anyhow::Result<model::ActualContainers> {
    let container_ids = command::stdout_utf8(process::Command::new("docker").args([
        "--host",
        docker_host,
        "compose",
        "ps",
        "--quiet",
    ]))?;
    let container_ids = container_ids.lines().collect::<Vec<_>>();

    let containers = if container_ids.is_empty() {
        vec![]
    } else {
        command::stdout_json(
            process::Command::new("docker")
                .args(["--host", docker_host, "inspect", "--"])
                .args(container_ids),
        )?
    };

    Ok(containers.into_iter().map(convert_container).collect())
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Container {
    config: Config,
    id: String,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "PascalCase")]
struct Config {
    labels: collections::BTreeMap<String, String>,
}

fn convert_container(container: Container) -> model::ActualContainer {
    model::ActualContainer {
        container_id: container.id,
        // TODO: Consider Podman Compose with `io.podman.compose.config-hash`.
        service_config_hash: container.config.labels["com.docker.compose.config-hash"].clone(),
        service_name: container.config.labels["com.docker.compose.service"].clone(),
    }
}
