use crate::library::clean;
use crate::library::configuration;
use crate::library::constants;
use crate::library::loop_until_sigint;
use crate::library::provision;
use crate::library::run_command;
use anyhow::Context;
use std::fs;
use std::process;

pub fn go() -> anyhow::Result<()> {
    let configuration = configuration::get()?;

    loop_until_sigint::go(loop_until_sigint::In {
        set_up: || set_up(&configuration),
        iterate: || iterate(&configuration),
        tear_down: || clean::go().expect("Unable to clean."),
    })
}

fn set_up(configuration: &configuration::Main) -> anyhow::Result<()> {
    set_up_work_folder()?;
    start_staging(configuration)?;
    dump_staging_ssh_configuration(configuration)?;
    provision_staging(configuration)
}

fn set_up_work_folder() -> anyhow::Result<()> {
    let path = constants::WORK_FOLDER;
    fs::create_dir(path)
        .with_context(|| format!("Unable to create folder, consider cleaning: {path}"))?;
    fs::write(constants::provision_base_file(), constants::PROVISION_BASE)?;
    fs::write(constants::vagrantfile_file(), constants::VAGRANTFILE)?;
    Ok(())
}

fn start_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    run_command::go(
        process::Command::new("vagrant")
            .arg("up")
            .current_dir(constants::WORK_FOLDER)
            .env("KEREK_IP", &configuration.staging.public_ip),
    )
}

fn dump_staging_ssh_configuration(configuration: &configuration::Main) -> anyhow::Result<()> {
    let file = fs::File::create(&configuration.staging.ssh_configuration_file)?;
    run_command::go(
        process::Command::new("vagrant")
            .arg("ssh-config")
            .stdout(file)
            .current_dir(constants::WORK_FOLDER),
    )
}

fn provision_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    provision::go(
        configuration,
        provision::In {
            ssh_configuration_file: &configuration.staging.ssh_configuration_file,
            ssh_host: &configuration.staging.ssh_host,
            kubeconfig_file: &configuration.staging.kubeconfig_file,
            public_ip: &configuration.staging.public_ip,
        },
    )
}

fn iterate(configuration: &configuration::Main) -> anyhow::Result<()> {
    run_command::go(&mut process::Command::new(&configuration.test_base))
}
