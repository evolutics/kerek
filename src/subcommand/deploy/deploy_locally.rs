use crate::library::command;
use crate::library::configuration;
use anyhow::Context;
use std::fs;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    let deploy_on_remote = tempfile::NamedTempFile::new()?;
    fs::write(&deploy_on_remote, include_str!("deploy_on_remote.py"))
        .context("Unable to write file: deploy_on_remote.py")?;

    command::status_ok(
        process::Command::new("python3")
            .arg(deploy_on_remote.as_ref())
            .env(
                "WHEELSTICKS_REMOTE_WORKBENCH",
                &configuration.x_wheelsticks.remote_workbench,
            ),
    )
}
