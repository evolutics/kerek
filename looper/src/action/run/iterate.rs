use crate::library::assets;
use crate::library::command;
use crate::library::configuration;
use std::path;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    run_base_test(configuration)?;
    build(configuration)?;
    deploy_staging(configuration)?;
    test_staging(configuration)?;
    deploy_production(configuration)?;
    test_production(configuration)
}

fn run_base_test(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(&mut process::Command::new(&configuration.base_test))
}

fn build(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("skaffold")
            .arg("build")
            .arg("--file-output")
            .arg(configuration.work_folder.join(assets::BUILD_FILENAME)),
    )
}

fn deploy_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    deploy(configuration, &configuration.staging.kubeconfig_file)
}

fn deploy(configuration: &configuration::Main, kubeconfig_file: &path::Path) -> anyhow::Result<()> {
    command::status(
        process::Command::new("skaffold")
            .arg("deploy")
            .arg("--build-artifacts")
            .arg(configuration.work_folder.join(assets::BUILD_FILENAME))
            .arg("--kubeconfig")
            .arg(kubeconfig_file),
    )
}

fn test_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.smoke_test).arg(&configuration.staging.public_ip),
    )?;
    command::status(
        process::Command::new(&configuration.acceptance_test).arg(&configuration.staging.public_ip),
    )
}

fn deploy_production(configuration: &configuration::Main) -> anyhow::Result<()> {
    deploy(configuration, &configuration.production.kubeconfig_file)
}

fn test_production(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.smoke_test).arg(&configuration.production.public_ip),
    )
}
