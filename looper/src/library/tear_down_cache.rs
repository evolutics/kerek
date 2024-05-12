use super::command;
use super::configuration;
use anyhow::Context;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Tearing down cache.");

    delete_cache_vm_if_exists(configuration)?;
    delete_cache_folder_if_exists(configuration)
}

fn delete_cache_vm_if_exists(configuration: &configuration::Main) -> anyhow::Result<()> {
    if configuration.cache.vagrantfile.exists() {
        command::status(
            process::Command::new("vagrant")
                .arg("destroy")
                .arg("--force")
                .current_dir(&configuration.cache.folder)
                .envs(&configuration.variables)
                .envs(&configuration.staging.variables),
        )
        .context("Unable to delete cache VM.")
    } else {
        Ok(())
    }
}

fn delete_cache_folder_if_exists(configuration: &configuration::Main) -> anyhow::Result<()> {
    let folder = &configuration.cache.folder;
    if folder.exists() {
        fs::remove_dir_all(folder)
            .with_context(|| format!("Unable to delete cache folder: {folder:?}"))?;
    }
    Ok(())
}
