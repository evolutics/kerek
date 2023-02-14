use crate::library::command;
use anyhow::Context;
use std::env;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let playbook = tempfile::NamedTempFile::new()?;
    fs::write(&playbook, include_str!("playbook.yaml"))
        .context("Unable to write file \"playbook.yaml\"")?;
    let provision_test = tempfile::NamedTempFile::new()?;
    fs::write(&provision_test, include_str!("provision_test.sh"))
        .context("Unable to write file \"provision_test.sh\"")?;

    let ssh_host = in_.ssh_host;

    command::status_ok(
        process::Command::new("ansible-playbook")
            .args([
                "--extra-vars",
                &serde_json::to_string(&PlaybookVariables {
                    deploy_user: &in_.deploy_user,
                    own_executable: &env::current_exe()
                        .context("Unable to get current executable")?,
                    provision_test: provision_test.as_ref(),
                    upgrade_packages: in_.upgrade_packages,
                })?,
                "--inventory",
                &format!(",{ssh_host}"),
            ])
            .args(in_.ssh_configuration.iter().flat_map(|ssh_configuration| {
                [
                    "--ssh-common-args".into(),
                    format!("-F {ssh_configuration:?}"),
                ]
            }))
            .args(
                in_.ssh_user
                    .iter()
                    .flat_map(|ssh_user| ["--user", ssh_user]),
            )
            .arg("--")
            .arg(playbook.as_ref()),
    )
}

pub struct In {
    pub deploy_user: String,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: String,
    pub ssh_user: Option<String>,
    pub upgrade_packages: bool,
}

#[derive(serde::Serialize)]
struct PlaybookVariables<'a> {
    deploy_user: &'a str,
    own_executable: &'a path::Path,
    provision_test: &'a path::Path,
    upgrade_packages: bool,
}
