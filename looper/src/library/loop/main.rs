use super::super::command;
use super::super::configuration;
use super::super::provision;
use super::super::set_up_cache;
use super::super::tear_down_cache;
use super::iterate;
use anyhow::Context;
use std::process;

pub fn go(configuration: &configuration::Main, is_dry_run: bool) -> anyhow::Result<()> {
    match load_vm_snapshot(configuration) {
        Err(_) => {
            crate::log!("No current VM snapshot exists, hence resetting.");
            tear_down_cache::go(configuration)?;
            set_up_cache::go(configuration, true)?;
            provision::go(configuration, &configuration.staging)?;
            save_vm_snapshot(configuration)?;
        }

        Ok(()) => crate::log!("Current VM snapshot loaded."),
    };

    for iteration in 0.. {
        crate::log!("Executing iteration number {iteration}.");
        iterate::go(configuration, is_dry_run)?;
        if is_dry_run {
            break;
        }
        load_vm_snapshot(configuration)?;
    }
    Ok(())
}

const VERSIONED_VM_SNAPSHOT_NAME: &str = env!("VERGEN_GIT_SHA");

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
