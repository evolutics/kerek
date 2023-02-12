use crate::library::command;
use crate::library::configuration;
use anyhow::Context;
use std::env;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let configuration = configuration::get(&in_.configuration)?;

    let playbook = tempfile::NamedTempFile::new()?;
    fs::write(&playbook, include_str!("playbook.yaml"))
        .context("Unable to write file: playbook.yaml")?;
    let provision_test = tempfile::NamedTempFile::new()?;
    fs::write(&provision_test, include_str!("provision_test.sh"))
        .context("Unable to write file: provision_test.sh")?;

    let ssh_host = in_.ssh_host;

    command::status_ok(
        process::Command::new("ansible-playbook")
            .arg("--inventory")
            .arg(format!(",{ssh_host}"))
            .args(in_.ssh_configuration.iter().flat_map(|ssh_configuration| {
                [
                    "--ssh-common-args".into(),
                    format!("-F {ssh_configuration:?}"),
                ]
            }))
            .arg("--")
            .arg(playbook.as_ref())
            .env(
                "WHEELSTICKS_DEPLOY_USER",
                configuration.x_wheelsticks.deploy_user,
            )
            .env(
                "WHEELSTICKS_EXECUTABLE",
                env::current_exe().context("Unable to get current executable.")?,
            )
            .env("WHEELSTICKS_PROVISION_TEST", provision_test.as_ref()),
    )
}

pub struct In {
    pub configuration: path::PathBuf,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: String,
}
