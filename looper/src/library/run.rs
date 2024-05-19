use super::command;
use super::configuration;
use super::set_up_cache;
use super::tear_down_cache;
use anyhow::Context;
use std::process;

pub fn go(configuration: &configuration::Main, options: Options) -> anyhow::Result<()> {
    if options.is_cache_reset {
        tear_down_cache::go(configuration)?;
        set_up_cache::go(configuration, true)?;
    }
    provision(configuration, &configuration.staging)?;

    build(configuration)?;
    deploy(configuration, &configuration.staging)?;
    run_env_tests(configuration, &configuration.staging)?;

    if !options.is_dry_run {
        deploy(configuration, &configuration.production)?;
        run_env_tests(configuration, &configuration.production)?;
    }

    Ok(())
}

pub struct Options {
    pub is_cache_reset: bool,
    pub is_dry_run: bool,
}

fn provision(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    let environment_id = &environment.id;
    crate::log!("Provisioning {environment_id} environment.");
    command::status(
        process::Command::new(&configuration.life_cycle.provision[0])
            .args(&configuration.life_cycle.provision[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| format!("Unable to provision {environment_id} environment"))
}

fn build(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Building.");
    command::status(
        process::Command::new(&configuration.life_cycle.build[0])
            .args(&configuration.life_cycle.build[1..])
            .envs(&configuration.variables),
    )
    .context("Unable to build")
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
    .with_context(|| format!("Unable to deploy to {environment_id} environment"))
}

fn run_env_tests(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    let environment_id = &environment.id;
    crate::log!("Running env tests for {environment_id} environment.");
    command::status(
        process::Command::new(&configuration.life_cycle.env_tests[0])
            .args(&configuration.life_cycle.env_tests[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| format!("Env tests failed for {environment_id} environment"))
}
