use crate::library::command;
use crate::library::configuration;
use anyhow::Context;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    run_base_tests(configuration)?;
    build(configuration)?;
    deploy(configuration, &configuration.staging)?;
    run_smoke_tests(configuration, &configuration.staging)?;
    run_acceptance_tests(configuration, &configuration.staging)?;
    deploy(configuration, &configuration.production)?;
    run_smoke_tests(configuration, &configuration.production)?;
    move_to_next_version(configuration)
}

fn run_base_tests(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.base[0]).args(&configuration.tests.base[1..]),
    )
    .context("Base tests failed.")
}

fn build(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.build[0])
            .args(&configuration.life_cycle.build[1..]),
    )
    .context("Unable to build.")
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
    .with_context(|| {
        let environment = &environment.display_name;
        format!("Unable to deploy to {environment}.")
    })
}

fn run_smoke_tests(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.smoke[0])
            .args(&configuration.tests.smoke[1..])
            .env("KEREK_IP_ADDRESS", &environment.ip_address),
    )
    .with_context(|| {
        let environment = &environment.display_name;
        format!("Smoke tests for {environment} failed.")
    })
}

fn run_acceptance_tests(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.acceptance[0])
            .args(&configuration.tests.acceptance[1..])
            .env("KEREK_IP_ADDRESS", &environment.ip_address),
    )
    .with_context(|| {
        let environment = &environment.display_name;
        format!("Acceptance tests for {environment} failed.")
    })
}

fn move_to_next_version(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.move_to_next_version[0])
            .args(&configuration.life_cycle.move_to_next_version[1..]),
    )
    .context("Unable to move to next version.")
}
