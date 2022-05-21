use crate::library::command;
use crate::library::configuration;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    run_base_tests(configuration)?;
    build(configuration)?;
    deploy(configuration, &configuration.staging)?;
    run_smoke_tests(configuration, &configuration.staging)?;
    run_acceptance_tests(configuration, &configuration.staging)?;
    deploy(configuration, &configuration.production)?;
    run_smoke_tests(configuration, &configuration.production)?;
    load_snapshot(configuration)?;
    move_to_next_version(configuration)
}

fn run_base_tests(configuration: &configuration::Main) -> anyhow::Result<()> {
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

fn deploy(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.deploy[0])
            .args(&configuration.life_cycle.deploy[1..])
            .env("KEREK_KUBECONFIG", &environment.kubeconfig_file)
            .env(
                "KEREK_SSH_CONFIGURATION",
                &environment.ssh_configuration_file,
            )
            .env("KEREK_SSH_HOST", &environment.ssh_host),
    )
}

fn run_smoke_tests(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.smoke[0])
            .args(&configuration.tests.smoke[1..])
            .env("KEREK_IP", &environment.public_ip),
    )
}

fn run_acceptance_tests(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.acceptance[0])
            .args(&configuration.tests.acceptance[1..])
            .env("KEREK_IP", &environment.public_ip),
    )
}

fn load_snapshot(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("snapshot")
            .arg("restore")
            .arg("--")
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
