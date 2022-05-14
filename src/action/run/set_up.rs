use crate::library::command;
use crate::library::configuration;
use crate::library::provision;
use crate::library::set_up_workspace;
use crate::library::tear_down_workspace;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    tear_down_workspace::go(&configuration.work_folder)?;
    set_up_workspace::go(&configuration.work_folder)?;
    start_staging(configuration)?;
    dump_staging_ssh_configuration(configuration)?;
    provision_staging(configuration)
}

fn start_staging(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(
        process::Command::new("vagrant")
            .arg("up")
            .current_dir(&configuration.work_folder)
            .env("KEREK_IP", &configuration.staging.public_ip),
    )
}

fn dump_staging_ssh_configuration(configuration: &configuration::Main) -> anyhow::Result<()> {
    let file = fs::File::create(&configuration.staging.ssh_configuration_file)?;
    command::status(
        process::Command::new("vagrant")
            .arg("ssh-config")
            .current_dir(&configuration.work_folder)
            .stdout(file),
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
