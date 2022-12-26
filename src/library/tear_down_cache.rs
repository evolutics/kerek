use super::command;
use super::configuration;
use anyhow::Context;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    remove_vm_if_exists(configuration)?;
    remove_cache_folder_if_exists(configuration)
}

fn remove_vm_if_exists(configuration: &configuration::Main) -> anyhow::Result<()> {
    if configuration.cache.staging.vagrantfile.exists() {
        command::status(
            process::Command::new("vagrant")
                .arg("destroy")
                .arg("--force")
                .current_dir(&configuration.cache.staging.folder)
                .envs(&configuration.variables)
                .envs(&configuration.staging.variables),
        )
    } else {
        Ok(())
    }
}

fn remove_cache_folder_if_exists(configuration: &configuration::Main) -> anyhow::Result<()> {
    let folder = &configuration.cache.folder;
    if folder.exists() {
        fs::remove_dir_all(folder)
            .with_context(|| format!("Unable to remove cache folder: {folder:?}"))?;
    }
    Ok(())
}
