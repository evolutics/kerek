use super::command;
use super::configuration;
use anyhow::Context;
use std::borrow;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Setting up cache.");

    create_cache_folder(configuration)?;
    start_cache_vm(configuration)?;
    dump_cache_vm_ssh_config(configuration)
}

fn create_cache_folder(configuration: &configuration::Main) -> anyhow::Result<()> {
    let folder = &configuration.cache.folder;
    fs::create_dir_all(folder)
        .with_context(|| format!("Unable to create cache folder: {folder:?}"))?;

    for (file, contents) in [
        (
            &configuration.cache.scripts,
            include_str!("assets/scripts.sh"),
        ),
        (
            &configuration.cache.vagrantfile,
            &get_vagrantfile_contents(configuration)?,
        ),
    ] {
        fs::write(file, contents)
            .with_context(|| format!("Unable to write cache file: {file:?}"))?;
    }

    Ok(())
}

fn get_vagrantfile_contents(
    configuration: &configuration::Main,
) -> anyhow::Result<borrow::Cow<str>> {
    Ok(match &configuration.vagrantfile {
        None => include_str!("assets/Vagrantfile").into(),
        Some(path) => {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Unable to read Vagrantfile: {path:?}"))?;
            contents.into()
        }
    })
}

fn start_cache_vm(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("up")
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables),
    )
    .context("Unable to start cache VM")
}

fn dump_cache_vm_ssh_config(configuration: &configuration::Main) -> anyhow::Result<()> {
    let path = &configuration.cache.ssh_config;
    let file = fs::File::create(path)
        .with_context(|| format!("Unable to create SSH config file: {path:?}"))?;
    command::status(
        process::Command::new("vagrant")
            .args(["ssh-config", "--host", &configuration.staging.id])
            .current_dir(&configuration.cache.folder)
            .envs(&configuration.variables)
            .envs(&configuration.staging.variables)
            .stdout(file),
    )
    .context("Unable to dump cache VM SSH config")
}
