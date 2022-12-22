use crate::library::command;
use crate::library::configuration;
use crate::library::provision;
use crate::library::set_up_cache;
use crate::library::tear_down_cache;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    tear_down_cache::go(configuration)?;
    set_up_cache::go(configuration)?;
    start_staging(configuration)?;
    dump_staging_ssh_configuration(configuration)?;
    provision_staging(configuration)
}

fn start_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("up")
            .current_dir(&configuration.cache.folder),
    )
}

fn dump_staging_ssh_configuration(configuration: &configuration::Main) -> anyhow::Result<()> {
    let file = fs::File::create(&configuration.staging.ssh_configuration_file)?;
    command::status(
        process::Command::new("vagrant")
            .arg("ssh-config")
            .current_dir(&configuration.cache.folder)
            .stdout(file),
    )
}

fn provision_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    provision::go(provision::In {
        script_file: &configuration.cache.provision,
        ssh_configuration_file: &configuration.staging.ssh_configuration_file,
        ssh_host: &configuration.staging.ssh_host,
    })
}
