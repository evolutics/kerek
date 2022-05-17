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
    test_production(configuration)?;
    load_snapshot(configuration)?;
    move_to_next_version(configuration)
}

fn run_base_test(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.base[0]).args(&configuration.tests.base[1..]),
    )
}

fn build(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.build[0])
            .args(&configuration.life_cycle.build[1..]),
    )
}

fn deploy_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    deploy(configuration, &configuration.staging.kubeconfig_file)
}

fn deploy(configuration: &configuration::Main, kubeconfig_file: &path::Path) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.deploy[0])
            .args(&configuration.life_cycle.deploy[1..])
            .env("KEREK_KUBECONFIG", kubeconfig_file),
    )
}

fn test_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.smoke[0])
            .args(&configuration.tests.smoke[1..])
            .env("KEREK_IP", &configuration.staging.public_ip),
    )?;
    command::status(
        process::Command::new(&configuration.tests.acceptance[0])
            .args(&configuration.tests.acceptance[1..])
            .env("KEREK_IP", &configuration.staging.public_ip),
    )
}

fn deploy_production(configuration: &configuration::Main) -> anyhow::Result<()> {
    deploy(configuration, &configuration.production.kubeconfig_file)
}

fn test_production(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.smoke[0])
            .args(&configuration.tests.smoke[1..])
            .env("KEREK_IP", &configuration.production.public_ip),
    )
}

fn load_snapshot(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("snapshot")
            .arg("restore")
            .arg(&configuration.cache.vm_snapshot)
            .current_dir(&configuration.cache.folder),
    )
}

fn move_to_next_version(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.move_to_next_version[0])
            .args(&configuration.life_cycle.move_to_next_version[1..]),
    )
}
