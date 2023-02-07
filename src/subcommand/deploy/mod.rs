use crate::library::command;
use crate::library::configuration;
use anyhow::Context;
use std::ffi;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let configuration = configuration::get(&in_.configuration)?;

    let deploy_on_remote = configuration
        .x_wheelsticks
        .local_workbench
        .join("deploy_on_remote.py");
    fs::write(deploy_on_remote, include_str!("deploy_on_remote.py"))
        .context("Unable to write file: deploy_on_remote.py")?;

    // TODO: Support deploying on same machine without SSH.

    let ssh_configuration = in_.ssh_configuration.unwrap_or_default();
    let ssh_host = in_.ssh_host.unwrap_or_default();

    println!("Synchronizing artifacts.");
    synchronize_artifacts(&configuration, &ssh_configuration, &ssh_host)?;
    println!("Deploying on remote.");
    run_deploy_on_remote(&configuration, &ssh_configuration, &ssh_host)
}

pub struct In {
    pub configuration: path::PathBuf,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: Option<String>,
}

fn synchronize_artifacts(
    configuration: &configuration::Main,
    ssh_configuration: &path::Path,
    ssh_host: &str,
) -> anyhow::Result<()> {
    let mut source = ffi::OsString::from(&configuration.x_wheelsticks.local_workbench);
    source.push("/");
    let source = source;

    let mut destination = ffi::OsString::from(&configuration.x_wheelsticks.deploy_user);
    destination.push("@");
    destination.push(ssh_host);
    destination.push(":");
    destination.push(&configuration.x_wheelsticks.remote_workbench);
    let destination = destination;

    command::status_ok(
        process::Command::new("rsync")
            .arg("--archive")
            .arg("--delete")
            .arg("--rsh")
            .arg(format!("ssh -F {ssh_configuration:?}"))
            .arg("--")
            .arg(source)
            .arg(destination),
    )
}

fn run_deploy_on_remote(
    configuration: &configuration::Main,
    ssh_configuration: &path::Path,
    ssh_host: &str,
) -> anyhow::Result<()> {
    let remote_workbench = &configuration.x_wheelsticks.remote_workbench;

    let mut remote_workbench_binding = ffi::OsString::from("WHEELSTICKS_REMOTE_WORKBENCH=");
    remote_workbench_binding.push(remote_workbench);
    let remote_workbench_binding = remote_workbench_binding;

    command::status_ok(
        process::Command::new("ssh")
            .arg("-F")
            .arg(ssh_configuration)
            .arg("-l")
            .arg(&configuration.x_wheelsticks.deploy_user)
            .arg(ssh_host)
            .arg("--")
            .arg(remote_workbench_binding)
            .arg("python3")
            .arg(remote_workbench.join("deploy_on_remote.py")),
    )
}
