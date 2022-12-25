use crate::library::command;
use crate::library::configuration;
use crate::library::provision;
use crate::library::set_up_cache;
use crate::library::tear_down_cache;
use anyhow::Context;
use std::ffi;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    tear_down_cache::go(configuration)?;
    set_up_cache::go(configuration)?;
    start_staging(configuration)?;
    dump_staging_ssh_configuration(configuration)?;
    provision::go(configuration, &configuration.staging)
}

fn start_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("up")
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables),
    )
}

fn dump_staging_ssh_configuration(configuration: &configuration::Main) -> anyhow::Result<()> {
    let path = &configuration.staging.variables[ffi::OsStr::new("KEREK_SSH_CONFIGURATION")];
    let file = fs::File::create(path)
        .with_context(|| format!("Unable to create SSH configuration file: {path:?}"))?;
    command::status(
        process::Command::new("vagrant")
            .arg("ssh-config")
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables)
            .stdout(file),
    )
}
