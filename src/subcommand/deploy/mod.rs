use crate::library::command;
use anyhow::Context;
use std::fs;
use std::path;
use std::process;

pub fn go(_configuration: path::PathBuf) -> anyhow::Result<()> {
    let deploy = tempfile::NamedTempFile::new()?;
    fs::write(&deploy, include_str!("deploy.py")).context("Unable to write file: deploy.py")?;
    let deploy_on_remote = tempfile::NamedTempFile::new()?;
    fs::write(&deploy_on_remote, include_str!("deploy_on_remote.py"))
        .context("Unable to write file: deploy_on_remote.py")?;

    command::status_ok(
        process::Command::new("python3")
            .arg("--")
            .arg(deploy.as_ref())
            .env("WHEELSTICKS_DEPLOY_ON_REMOTE", deploy_on_remote.as_ref()),
    )
}
