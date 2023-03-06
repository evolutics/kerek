mod deploy_locally;

use crate::library::command;
use crate::library::compose;
use anyhow::Context;
use std::ffi;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let project = compose::parse(compose::Parameters {
        compose_file: &in_.compose_file,
        environment_files: in_.environment_files,
        project_folder: in_.project_folder,
        project_name: in_.project_name,
    })?;

    match in_.ssh_host {
        None => deploy_locally::go(&project),
        Some(ssh_host) => deploy_remotely(
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
    pub environment_files: Option<Vec<path::PathBuf>>,
    pub project_folder: Option<path::PathBuf>,
    pub project_name: Option<String>,
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: Option<String>,
    pub ssh_user: Option<String>,
}

struct Ssh {
    configuration: Option<path::PathBuf>,
    host: String,
    user: Option<String>,
}

fn deploy_remotely(project: &compose::Project, ssh: &Ssh) -> anyhow::Result<()> {
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
            .arg(path::Path::new(&project.x_wheelsticks.remote_workbench).join("compose.yaml"))
            .arg("--env-file"),
    )
}
