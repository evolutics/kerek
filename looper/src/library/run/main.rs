use super::super::command;
use super::super::configuration;
use super::iterate;
use super::reset;
use std::process;

pub fn go(configuration: &configuration::Main, is_dry_run: bool) -> anyhow::Result<()> {
    load_snapshot(configuration).or_else(|_| {
        reset::go(configuration)?;
        save_snapshot(configuration)
    })?;

    loop {
        iterate::go(configuration, is_dry_run)?;
        if is_dry_run {
            break Ok(());
        }
        load_snapshot(configuration)?;
    }
}

const VERSIONED_SNAPSHOT_NAME: &str = env!("VERGEN_GIT_SHA");

fn load_snapshot(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("snapshot")
            .arg("restore")
            .arg("--")
            .arg(VERSIONED_SNAPSHOT_NAME)
            .current_dir(&configuration.cache.staging.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables),
    )
}

fn save_snapshot(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("snapshot")
            .arg("save")
            .arg("--force")
            .arg("--")
            .arg(VERSIONED_SNAPSHOT_NAME)
            .current_dir(&configuration.cache.staging.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables),
    )
}
