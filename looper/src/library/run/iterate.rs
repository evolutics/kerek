use super::super::command;
use super::super::configuration;
use anyhow::Context;
use std::process;

pub fn go(configuration: &configuration::Main, is_dry_run: bool) -> anyhow::Result<()> {
    run_base_tests(configuration)?;
    build(configuration)?;
    deploy(configuration, &configuration.staging)?;
    run_smoke_tests(configuration, &configuration.staging)?;
    run_acceptance_tests(configuration, &configuration.staging)?;
    if !is_dry_run {
        deploy(configuration, &configuration.production)?;
        run_smoke_tests(configuration, &configuration.production)?;
        move_to_next_version(configuration)?;
    }
    Ok(())
}

fn run_base_tests(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.base[0])
            .args(&configuration.tests.base[1..])
            .envs(&configuration.variables),
    )
    .context("Base tests failed.")
}

fn build(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.build[0])
            .args(&configuration.life_cycle.build[1..])
            .envs(&configuration.variables),
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
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| {
        let environment = &environment.id;
        format!("Unable to deploy to environment {environment}.")
    })
}

fn run_smoke_tests(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.smoke[0])
            .args(&configuration.tests.smoke[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| {
        let environment = &environment.id;
        format!("Smoke tests failed for environment {environment}.")
    })
}

fn run_acceptance_tests(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.tests.acceptance[0])
            .args(&configuration.tests.acceptance[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| {
        let environment = &environment.id;
        format!("Acceptance tests failed for environment {environment}.")
    })
}

fn move_to_next_version(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.move_to_next_version[0])
            .args(&configuration.life_cycle.move_to_next_version[1..])
            .envs(&configuration.variables),
    )
    .context("Unable to move to next version.")
}
