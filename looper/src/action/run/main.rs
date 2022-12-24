use super::iterate;
use super::reset;
use crate::library::command;
use crate::library::configuration;
use std::path;
use std::process;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;

    load_snapshot(&configuration).or_else(|_| {
        reset::go(&configuration)?;
        save_snapshot(&configuration)
    })?;

    loop {
        iterate::go(&configuration)?;
        load_snapshot(&configuration)?;
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
            .current_dir(&configuration.cache.folder)
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
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.staging.variables),
    )
}
