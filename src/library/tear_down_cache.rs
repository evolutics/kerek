use super::command;
use super::configuration;
use std::fs;
use std::process;

pub fn go(cache: &configuration::Cache) -> anyhow::Result<()> {
    remove_vm_if_exists(cache)?;
    remove_cache_folder_if_exists(cache)
}

fn remove_vm_if_exists(cache: &configuration::Cache) -> anyhow::Result<()> {
    if cache.vagrantfile.exists() {
        command::status(
            process::Command::new("vagrant")
                .arg("destroy")
                .arg("--force")
                .current_dir(&cache.folder),
        )
    } else {
        Ok(())
    }
}

fn remove_cache_folder_if_exists(cache: &configuration::Cache) -> anyhow::Result<()> {
    if cache.folder.exists() {
        fs::remove_dir_all(&cache.folder)?;
    }
    Ok(())
}
