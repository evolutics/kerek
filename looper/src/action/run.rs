use crate::library::clean;
use crate::library::configuration;
use crate::library::constants;
use crate::library::loop_until_sigint;
use crate::library::run_command;
use anyhow::Context;
use std::fs;
use std::process;

pub fn go() -> anyhow::Result<()> {
    let configuration = configuration::get()?;

    loop_until_sigint::go(
        set_up,
        || iterate(&configuration),
        || clean::go().expect("Unable to clean."),
    )
}

fn set_up() -> anyhow::Result<()> {
    set_up_work_folder()?;
    start_staging_vm()?;
    provision_staging_vm()
}

fn set_up_work_folder() -> anyhow::Result<()> {
    let path = constants::WORK_FOLDER;
    fs::create_dir(path)
        .with_context(|| format!("Unable to create folder, consider cleaning: {path}"))?;
    fs::write(constants::provision_base_file(), constants::PROVISION_BASE)?;
    fs::write(constants::vagrantfile_file(), constants::VAGRANTFILE)?;
    Ok(())
}

fn start_staging_vm() -> anyhow::Result<()> {
    run_command::go(
        process::Command::new("vagrant")
            .arg("up")
            .current_dir(constants::WORK_FOLDER)
            .env("KEREK_IP", constants::STAGING_IP),
    )
}

fn provision_staging_vm() -> anyhow::Result<()> {
    dump_ssh_configuration()
}

fn dump_ssh_configuration() -> anyhow::Result<()> {
    let file = fs::File::create(constants::ssh_configuration_file())?;
    run_command::go(
        process::Command::new("vagrant")
            .arg("ssh-config")
            .stdout(file)
            .current_dir(constants::WORK_FOLDER),
    )
}

fn iterate(configuration: &configuration::Data) -> anyhow::Result<()> {
    eprintln!("{configuration:#?}");
    Ok(())
}
