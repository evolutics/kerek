mod deploy_locally;

use crate::library::command;
use crate::library::compose;
use anyhow::Context;
use std::ffi;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let project = compose::parse(&in_.compose_file)?;

    match in_.ssh_host {
        None => deploy_locally::go(&project),
        Some(ssh_host) => deploy_remotely(
            &in_.compose_file,
            &project,
            &Ssh {
                configuration: in_.ssh_configuration,
                host: ssh_host,
                user: in_.ssh_user,
            },
        ),
    }
}

pub struct In {
    pub compose_file: path::PathBuf,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: Option<String>,
    pub ssh_user: Option<String>,
}

struct Ssh {
    configuration: Option<path::PathBuf>,
    host: String,
    user: Option<String>,
}

fn deploy_remotely(
    compose_file: &path::Path,
    project: &compose::Project,
    ssh: &Ssh,
) -> anyhow::Result<()> {
    println!("Assembling artifacts.");
    assemble_artifacts(compose_file, project)?;
    println!("Synchronizing artifacts.");
    synchronize_artifacts(project, ssh)?;
    println!("Deploying on remote.");
    run_deploy_on_remote(project, ssh)
}

fn assemble_artifacts(compose_file: &path::Path, project: &compose::Project) -> anyhow::Result<()> {
    let source = compose_file;
    let destination = path::Path::new(&project.x_wheelsticks.local_workbench).join("compose.yaml");
    let _ = fs::copy(source, &destination)
        .with_context(|| format!("Unable to copy file {source:?} to {destination:?}"))?;
    Ok(())
}

fn synchronize_artifacts(project: &compose::Project, ssh: &Ssh) -> anyhow::Result<()> {
    let local_workbench = &project.x_wheelsticks.local_workbench;
    let source = format!("{local_workbench}/");

    let optional_user = ssh
        .user
        .as_ref()
        .map(|user| format!("{user}@"))
        .unwrap_or_default();
    let host = &ssh.host;
    let remote_workbench = &project.x_wheelsticks.remote_workbench;
    let destination = format!("{optional_user}{host}:{remote_workbench}");

    command::status_ok(
        process::Command::new("rsync")
            .args(["--archive", "--delete"])
            .args(
                ssh.configuration.iter().flat_map(|configuration| {
                    ["--rsh".into(), format!("ssh -F {configuration:?}")]
                }),
            )
            .arg("--")
            .args([source, destination]),
    )
}

fn run_deploy_on_remote(project: &compose::Project, ssh: &Ssh) -> anyhow::Result<()> {
    command::status_ok(
        process::Command::new("ssh")
            .args(
                ssh.configuration
                    .iter()
                    .flat_map(|configuration| [ffi::OsStr::new("-F"), configuration.as_os_str()]),
            )
            .args(ssh.user.iter().flat_map(|user| ["-l", user]))
            .args([&ssh.host, "--", "wheelsticks", "deploy", "--compose-file"])
            .arg(path::Path::new(&project.x_wheelsticks.remote_workbench).join("compose.yaml")),
    )
}
