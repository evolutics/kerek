use super::super::command;
use super::super::configuration;
use super::super::provision;
use super::super::set_up_cache;
use super::super::tear_down_cache;
use super::iterate;
use anyhow::Context;
use std::cmp;
use std::process;

pub fn go(configuration: &configuration::Main, mode: Mode) -> anyhow::Result<()> {
    for iteration in 0.. {
        crate::log!("Executing iteration number {iteration}.");

        reset(configuration, iteration == 0)?;

        iterate::go(configuration, mode == Mode::DryRunOnce)?;
        if mode != Mode::Loop {
            break;
        }

        move_to_next_version(configuration)?;
    }
    Ok(())
}

#[derive(cmp::PartialEq)]
pub enum Mode {
    DryRunOnce,
    Loop,
    RunOnce,
}

const VERSIONED_VM_SNAPSHOT_NAME: &str = env!("VERGEN_GIT_SHA");

fn reset(configuration: &configuration::Main, is_first_iteration: bool) -> anyhow::Result<()> {
    match load_vm_snapshot(configuration) {
        Err(_) if is_first_iteration => {
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

fn move_to_next_version(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Moving to next version.");
    command::status(
        process::Command::new(&configuration.life_cycle.move_to_next_version[0])
            .args(&configuration.life_cycle.move_to_next_version[1..])
            .envs(&configuration.variables),
    )
    .context("Unable to move to next version.")
}
