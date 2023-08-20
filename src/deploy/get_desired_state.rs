use super::model;
use crate::command;
use crate::docker;
use std::collections;

pub fn go(docker_cli: &docker::Cli) -> anyhow::Result<model::DesiredState> {
    let compose_app_definition = get_compose_app_definition(docker_cli)?;
    let service_config_hashes = get_service_config_hashes(docker_cli)?;

    Ok(model::DesiredState {
        project_name: compose_app_definition.name,
        services: compose_app_definition
            .services
            .into_iter()
            .map(|(service_name, service_definition)| {
                let service_config_hash = service_config_hashes[&service_name].clone();
                (
                    service_name,
                    convert_service_definition(service_definition, service_config_hash),
                )
            })
            .collect(),
    })
}

#[derive(serde::Deserialize)]
struct ComposeAppDefinition {
    name: String,
    services: collections::BTreeMap<String, ServiceDefinition>,
}

#[derive(serde::Deserialize)]
struct ServiceDefinition {
    deploy: Option<Deploy>,
}

#[derive(serde::Deserialize)]
struct Deploy {
    replicas: Option<u16>,
    update_config: Option<UpdateConfig>,
}

#[derive(serde::Deserialize)]
struct UpdateConfig {
    order: Option<OperationOrder>,
}

#[derive(serde::Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum OperationOrder {
    StartFirst,
    StopFirst,
}

fn get_compose_app_definition(docker_cli: &docker::Cli) -> anyhow::Result<ComposeAppDefinition> {
    command::stdout_json(
        docker_cli
            .docker_compose()
            .args(["config", "--format", "json"]),
    )
}

fn get_service_config_hashes(
    docker_cli: &docker::Cli,
) -> anyhow::Result<collections::BTreeMap<String, String>> {
    let service_hashes =
        command::stdout_utf8(docker_cli.docker_compose().args(["config", "--hash", "*"]))?;

    Ok(service_hashes
        .lines()
        .map(|line| {
            let key_value = line.split_whitespace().collect::<Vec<_>>();
            (key_value[0].into(), key_value[1].into())
        })
        .collect())
}

fn convert_service_definition(
    service_definition: ServiceDefinition,
    service_config_hash: String,
) -> model::DesiredServiceDefinition {
    model::DesiredServiceDefinition {
        replica_count: service_definition
            .deploy
            .as_ref()
            .and_then(|deploy| deploy.replicas)
            .unwrap_or(1),
        service_config_hash,
        update_order: match service_definition
            .deploy
            .and_then(|deploy| deploy.update_config)
            .and_then(|update_config| update_config.order)
            .unwrap_or(OperationOrder::StopFirst)
        {
            OperationOrder::StartFirst => model::OperationOrder::StartFirst,
            OperationOrder::StopFirst => model::OperationOrder::StopFirst,
        },
    }
}
