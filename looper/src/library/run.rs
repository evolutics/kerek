use super::command;
use super::configuration;
use super::provision;
use super::set_up_cache;
use super::tear_down_cache;
use anyhow::Context;
use std::process;

pub fn go(configuration: &configuration::Main, options: Options) -> anyhow::Result<()> {
    reset(configuration, options.is_vm_snapshot_asserted)?;

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
    pub is_dry_run: bool,
    pub is_vm_snapshot_asserted: bool,
}

const VERSIONED_VM_SNAPSHOT_NAME: &str = env!("VERGEN_GIT_SHA");

fn reset(configuration: &configuration::Main, is_vm_snapshot_asserted: bool) -> anyhow::Result<()> {
    match load_vm_snapshot(configuration) {
        Err(_) if !is_vm_snapshot_asserted => {
            crate::log!("No current VM snapshot exists, hence resetting from scratch.");
            tear_down_cache::go(configuration)?;
            set_up_cache::go(configuration, true)?;
            provision::go(configuration, &configuration.staging)?;
            save_vm_snapshot(configuration)
        }
        result => result,
    }
}

fn load_vm_snapshot(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Loading VM snapshot: {VERSIONED_VM_SNAPSHOT_NAME:?}");
    command::status(
        process::Command::new("vagrant")
            .arg("snapshot")
            .arg("restore")
            .arg("--")
            .arg(VERSIONED_VM_SNAPSHOT_NAME)
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables),
    )
    .with_context(|| format!("Unable to load VM snapshot: {VERSIONED_VM_SNAPSHOT_NAME:?}"))
}

fn save_vm_snapshot(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Saving VM snapshot: {VERSIONED_VM_SNAPSHOT_NAME:?}");
    command::status(
        process::Command::new("vagrant")
            .arg("snapshot")
            .arg("save")
            .arg("--force")
            .arg("--")
            .arg(VERSIONED_VM_SNAPSHOT_NAME)
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables),
    )
    .with_context(|| format!("Unable to save VM snapshot: {VERSIONED_VM_SNAPSHOT_NAME:?}"))
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
        process::Command::new(&configuration.life_cycle.env_tests[0])
            .args(&configuration.life_cycle.env_tests[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| format!("Env tests failed for {environment_id} environment."))
}
