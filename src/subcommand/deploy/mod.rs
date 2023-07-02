mod deploy_locally;

use crate::library::command;
use crate::library::compose;
use crate::library::docker_host;
use anyhow::Context;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let docker_host = docker_host::get(in_.docker_host)?;
    let project = compose::parse(compose::Parameters {
        compose_file: &in_.compose_file,
        environment_files: in_.environment_files,
        project_folder: in_.project_folder,
        project_name: in_.project_name,
    })?;

    match docker_host.ssh {
        None => deploy_locally::go(&project),
        Some(ssh) => deploy_remotely(&project, &ssh),
    }
}

pub struct In {
    pub compose_file: path::PathBuf,
    pub docker_host: Option<String>,
    pub environment_files: Option<Vec<path::PathBuf>>,
    pub project_folder: Option<path::PathBuf>,
    pub project_name: Option<String>,
}

fn deploy_remotely(project: &compose::Project, ssh: &docker_host::Ssh) -> anyhow::Result<()> {
    eprintln!("Assembling artifacts.");
    assemble_artifacts(project)?;
    eprintln!("Synchronizing artifacts.");
    synchronize_artifacts(project, ssh)?;
    eprintln!("Deploying on remote.");
    run_deploy_on_remote(project, ssh)
}

fn assemble_artifacts(project: &compose::Project) -> anyhow::Result<()> {
    let contents = compose::print(project.clone()).context("Unable to print Compose file")?;
    let file = path::Path::new(&project.x_wheelsticks.local_workbench).join("compose.yaml");
    fs::write(&file, contents).with_context(|| format!("Unable to write Compose file to {file:?}"))
}

fn synchronize_artifacts(project: &compose::Project, ssh: &docker_host::Ssh) -> anyhow::Result<()> {
    let local_workbench = &project.x_wheelsticks.local_workbench;
    let source = format!("{local_workbench}/");

    let optional_user = ssh
        .user
        .as_ref()
        .map(|user| format!("{user}@"))
        .unwrap_or_default();
    let host = &ssh.hostname;
    let remote_workbench = &project.x_wheelsticks.remote_workbench;
    let destination = format!("{optional_user}{host}:{remote_workbench}");

    command::status_ok(
        process::Command::new("rsync")
            .args(["--archive", "--delete"])
            .args(
                ssh.port
                    .iter()
                    .flat_map(|port| ["--rsh".into(), format!("ssh -p {port:?}")]),
            )
            .arg("--")
            .args([source, destination]),
    )
}

fn run_deploy_on_remote(project: &compose::Project, ssh: &docker_host::Ssh) -> anyhow::Result<()> {
    let optional_user = ssh
        .user
        .as_ref()
        .map(|user| format!("{user}@"))
        .unwrap_or_default();
    let host = &ssh.hostname;
    let optional_port = ssh.port.map(|port| format!(":{port}")).unwrap_or_default();
    let destination = format!("ssh://{optional_user}{host}{optional_port}");

    command::status_ok(
        process::Command::new("ssh")
            .args([
                &destination,
                "--",
                "wheelsticks",
                "deploy",
                "--compose-file",
            ])
            .arg(path::Path::new(&project.x_wheelsticks.remote_workbench).join("compose.yaml"))
            .arg("--env-file"),
    )
}
