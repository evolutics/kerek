use super::super::command;
use super::super::configuration;
use anyhow::Context;
use std::process;

pub fn go(configuration: &configuration::Main, is_dry_run: bool) -> anyhow::Result<()> {
    build(configuration)?;
    deploy(configuration, &configuration.staging)?;
    run_env_tests(configuration, &configuration.staging)?;
    if !is_dry_run {
        deploy(configuration, &configuration.production)?;
        run_env_tests(configuration, &configuration.production)?;
        move_to_next_version(configuration)?;
    }
    Ok(())
}

fn build(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Building.");
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
    let environment_id = &environment.id;
    crate::log!("Deploying to {environment_id} environment.");
    command::status(
        process::Command::new(&configuration.life_cycle.deploy[0])
            .args(&configuration.life_cycle.deploy[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| format!("Unable to deploy to {environment_id} environment."))
}

fn run_env_tests(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    let environment_id = &environment.id;
    crate::log!("Running env tests for {environment_id} environment.");
    command::status(
        process::Command::new(&environment.env_tests[0])
            .args(&environment.env_tests[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| format!("Env tests failed for {environment_id} environment."))
}

fn move_to_next_version(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Moving to next version.");
    command::status(
        process::Command::new(&configuration.life_cycle.move_to_next_version[0])
            .args(&configuration.life_cycle.move_to_next_version[1..])
            .envs(&configuration.variables),
    )
    .context("Unable to move to next version.")
}
