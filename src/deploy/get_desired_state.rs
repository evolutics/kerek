use super::model;
use crate::command;
use crate::docker_compose;
use std::collections;

pub fn go(
    service_names: &collections::BTreeSet<String>,
    docker_compose_cli: &docker_compose::Cli,
    no_deps: bool,
) -> anyhow::Result<model::DesiredServices> {
    let compose_app_definition = get_compose_app_definition(service_names, docker_compose_cli)?;
    let service_config_hashes = get_service_config_hashes(docker_compose_cli)?;

    Ok(compose_app_definition
        .services
        .into_iter()
        .filter(|(service_name, _)| !no_deps || service_names.contains(service_name))
        .map(|(service_name, service_definition)| {
            let service_config_hash = service_config_hashes[&service_name].clone();
            (
                service_name,
                convert_service_definition(service_definition, service_config_hash),
            )
        })
        .collect())
}

#[derive(serde::Deserialize)]
struct ComposeAppDefinition {
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
enum OperationOrder {
    StartFirst,
    StopFirst,
}

fn get_compose_app_definition(
    service_names: &collections::BTreeSet<String>,
    docker_compose_cli: &docker_compose::Cli,
) -> anyhow::Result<ComposeAppDefinition> {
    command::stdout_json(
        docker_compose_cli
            .command()
            .args(["config", "--format", "json", "--"])
            .args(service_names),
    )
}

fn get_service_config_hashes(
    docker_compose_cli: &docker_compose::Cli,
) -> anyhow::Result<collections::BTreeMap<String, String>> {
    let service_hashes =
        command::stdout_table(docker_compose_cli.command().args(["config", "--hash", "*"]))?;

    Ok(service_hashes
        .into_iter()
        .map(|[service_name, service_config_hash]| (service_name, service_config_hash))
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
