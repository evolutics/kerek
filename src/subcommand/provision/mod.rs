use crate::library::command;
use anyhow::Context;
use std::fs;
use std::process;

pub fn go() -> anyhow::Result<()> {
    let playbook = tempfile::NamedTempFile::new()?;
    fs::write(&playbook, include_str!("playbook.yaml"))
        .context("Unable to write file: playbook.yaml")?;
    let provision = tempfile::NamedTempFile::new()?;
    fs::write(&provision, include_str!("provision.py"))
        .context("Unable to write file: provision.py")?;
    let provision_test = tempfile::NamedTempFile::new()?;
    fs::write(&provision_test, include_str!("provision_test.sh"))
        .context("Unable to write file: provision_test.sh")?;

    command::status_ok(
        process::Command::new("python3")
            .arg("--")
            .arg(provision.as_ref())
            .env("WHEELSTICKS_PLAYBOOK", playbook.as_ref())
            .env("WHEELSTICKS_PROVISION_TEST", provision_test.as_ref()),
    )
}
