use crate::library::assets;
use crate::library::clean;
use crate::library::configuration;
use crate::library::loop_until_sigint;
use crate::library::provision;
use crate::library::run_command;
use anyhow::Context;
use std::fs;
use std::path;
use std::process;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;

    loop_until_sigint::go(loop_until_sigint::In {
        set_up: || set_up(&configuration),
        iterate: || iterate(&configuration),
        tear_down: || clean::go(&configuration.work_folder).expect("Unable to clean."),
    })
}

fn set_up(configuration: &configuration::Main) -> anyhow::Result<()> {
    set_up_work_folder(&configuration.work_folder)?;
    start_staging(configuration)?;
    dump_staging_ssh_configuration(configuration)?;
    provision_staging(configuration)
}

fn set_up_work_folder(work_folder: &path::Path) -> anyhow::Result<()> {
    fs::create_dir(work_folder)
        .with_context(|| format!("Unable to create folder, consider cleaning: {work_folder:?}"))?;
    for (filename, contents) in [
        (assets::PROVISION_BASE_FILENAME, assets::PROVISION_BASE),
        (assets::VAGRANTFILE_FILENAME, assets::VAGRANTFILE),
    ] {
        fs::write(work_folder.join(filename), contents)?;
    }
    Ok(())
}

fn start_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    run_command::go(
        process::Command::new("vagrant")
            .arg("up")
            .current_dir(&configuration.work_folder)
            .env("KEREK_IP", &configuration.staging.public_ip),
    )
}

fn dump_staging_ssh_configuration(configuration: &configuration::Main) -> anyhow::Result<()> {
    let file = fs::File::create(&configuration.staging.ssh_configuration_file)?;
    run_command::go(
        process::Command::new("vagrant")
            .arg("ssh-config")
            .stdout(file)
            .current_dir(&configuration.work_folder),
    )
}

fn provision_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    provision::go(provision::In {
        scripts: &configuration.provisioning_scripts,
        ssh_configuration_file: &configuration.staging.ssh_configuration_file,
        ssh_host: &configuration.staging.ssh_host,
        kubeconfig_file: &configuration.staging.kubeconfig_file,
        public_ip: &configuration.staging.public_ip,
    })
}

fn iterate(configuration: &configuration::Main) -> anyhow::Result<()> {
    run_command::go(&mut process::Command::new(&configuration.base_test))
}
