use crate::library::command;
use crate::library::docker_host;
use anyhow::Context;
use std::env;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let docker_host = docker_host::get(in_.docker_host)?;
    let ssh = docker_host.ssh.ok_or_else(|| {
        let url = docker_host.url;
        anyhow::anyhow!(
            "Docker host can only be provisioned via SSH \
            but URL is {url:?}"
        )
    })?;

    let playbook = tempfile::NamedTempFile::new()?;
    fs::write(&playbook, include_str!("playbook.yaml"))
        .context("Unable to write file \"playbook.yaml\"")?;
    let provision_test = tempfile::NamedTempFile::new()?;
    fs::write(&provision_test, include_str!("provision_test.sh"))
        .context("Unable to write file \"provision_test.sh\"")?;

    let ssh_host = &ssh.hostname;

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
            .args(ssh.user.iter().flat_map(|user| ["--user", user]))
            .arg("--")
            .arg(playbook.as_ref())
            .envs(
                ssh.port
                    .iter()
                    .flat_map(|port| [("ANSIBLE_REMOTE_PORT", port.to_string())]),
            ),
    )
}

pub struct In {
    pub deploy_user: String,
    pub docker_host: Option<String>,
    pub upgrade_packages: bool,
}

#[derive(serde::Serialize)]
struct PlaybookVariables<'a> {
    deploy_user: &'a str,
    own_executable: &'a path::Path,
    provision_test: &'a path::Path,
    upgrade_packages: bool,
}
