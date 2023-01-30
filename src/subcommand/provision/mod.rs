use crate::library::command;
use crate::library::configuration;
use anyhow::Context;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let configuration = configuration::get(&in_.configuration)?;

    let playbook = tempfile::NamedTempFile::new()?;
    fs::write(&playbook, include_str!("playbook.yaml"))
        .context("Unable to write file: playbook.yaml")?;
    let provision = tempfile::NamedTempFile::new()?;
    fs::write(&provision, include_str!("provision.py"))
        .context("Unable to write file: provision.py")?;
    let provision_test = tempfile::NamedTempFile::new()?;
    fs::write(&provision_test, include_str!("provision_test.sh"))
        .context("Unable to write file: provision_test.sh")?;

    // TODO: Support provisioning on same machine without SSH.

    command::status_ok(
        process::Command::new("python3")
            .arg("--")
            .arg(provision.as_ref())
            .env(
                "WHEELSTICKS_DEPLOY_USER",
                configuration.x_wheelsticks.deploy_user,
            )
            .env("WHEELSTICKS_PLAYBOOK", playbook.as_ref())
            .env("WHEELSTICKS_PROVISION_TEST", provision_test.as_ref())
            .env(
                "WHEELSTICKS_SSH_CONFIGURATION",
                in_.ssh_configuration.unwrap_or_default(),
            )
            .env("WHEELSTICKS_SSH_HOST", in_.ssh_host.unwrap_or_default()),
    )
}

pub struct In {
    pub configuration: path::PathBuf,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: Option<String>,
}
