use super::command;
use super::configuration;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    remove_vm_if_exists(configuration)?;
    remove_cache_folder_if_exists(configuration)
}

fn remove_vm_if_exists(configuration: &configuration::Main) -> anyhow::Result<()> {
    if configuration.cache.vagrantfile.exists() {
        command::status(
            process::Command::new("vagrant")
                .arg("destroy")
                .arg("--force")
                .current_dir(&configuration.cache.folder)
                .envs(&configuration.variables)
                .envs(&configuration.staging.variables),
        )
    } else {
        Ok(())
    }
}

fn remove_cache_folder_if_exists(configuration: &configuration::Main) -> anyhow::Result<()> {
    if configuration.cache.folder.exists() {
        fs::remove_dir_all(&configuration.cache.folder)?;
    }
    Ok(())
}
