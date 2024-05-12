use super::super::command;
use super::super::configuration;
use super::super::provision;
use super::super::set_up_cache;
use super::super::tear_down_cache;
use anyhow::Context;
use std::ffi;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    tear_down_cache::go(configuration)?;
    set_up_cache::go(configuration)?;
    start_cache_vm(configuration)?;
    dump_cache_vm_ssh_configuration(configuration)?;
    provision::go(configuration, &configuration.staging)
}

fn start_cache_vm(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("up")
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables),
    )
}

fn dump_cache_vm_ssh_configuration(configuration: &configuration::Main) -> anyhow::Result<()> {
    let path = &configuration.cache.ssh_configuration;
    let file = fs::File::create(path)
        .with_context(|| format!("Unable to create SSH configuration file: {path:?}"))?;
    command::status(
        process::Command::new("vagrant")
            .arg("ssh-config")
            .arg("--host")
            .arg(&configuration.staging.variables[ffi::OsStr::new("KEREK_SSH_HOST")])
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables)
            .stdout(file),
    )
}
