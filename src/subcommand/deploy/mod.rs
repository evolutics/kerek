mod deploy_locally;

use crate::library::command;
use crate::library::configuration;
use anyhow::Context;
use std::ffi;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let configuration = configuration::get(&in_.configuration)?;

    match in_.ssh_host {
        None => deploy_locally::go(&configuration),
        Some(ssh_host) => deploy_remotely(
            &in_.configuration,
            &configuration,
            &in_.ssh_configuration,
            &ssh_host,
        ),
    }
}

pub struct In {
    pub configuration: path::PathBuf,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: Option<String>,
}

fn deploy_remotely(
    configuration_path: &path::Path,
    configuration: &configuration::Main,
    ssh_configuration: &Option<path::PathBuf>,
    ssh_host: &str,
) -> anyhow::Result<()> {
    println!("Assembling artifacts.");
    assemble_artifacts(configuration_path, configuration)?;
    println!("Synchronizing artifacts.");
    synchronize_artifacts(configuration, ssh_configuration, ssh_host)?;
    println!("Deploying on remote.");
    run_deploy_on_remote(configuration, ssh_configuration, ssh_host)
}

fn assemble_artifacts(
    configuration_path: &path::Path,
    configuration: &configuration::Main,
) -> anyhow::Result<()> {
    let source = configuration_path;
    let destination = configuration
        .x_wheelsticks
        .local_workbench
        .join("compose.yaml");
    let _ = fs::copy(source, &destination)
        .with_context(|| format!("Unable to copy file {source:?} to {destination:?}"))?;
    Ok(())
}

fn synchronize_artifacts(
    configuration: &configuration::Main,
    ssh_configuration: &Option<path::PathBuf>,
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
            .args(ssh_configuration.iter().flat_map(|ssh_configuration| {
                ["--rsh".into(), format!("ssh -F {ssh_configuration:?}")]
            }))
            .arg("--")
            .arg(source)
            .arg(destination),
    )
}

fn run_deploy_on_remote(
    configuration: &configuration::Main,
    ssh_configuration: &Option<path::PathBuf>,
    ssh_host: &str,
) -> anyhow::Result<()> {
    command::status_ok(
        process::Command::new("ssh")
            .args(ssh_configuration.iter().flat_map(|ssh_configuration| {
                [ffi::OsStr::new("-F"), ssh_configuration.as_os_str()]
            }))
            .arg("-l")
            .arg(&configuration.x_wheelsticks.deploy_user)
            .arg(ssh_host)
            .arg("--")
            .arg("wheelsticks")
            .arg("deploy")
            .arg("--configuration")
            .arg(
                configuration
                    .x_wheelsticks
                    .remote_workbench
                    .join("compose.yaml"),
            ),
    )
}
