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
        project_folder: in_.project_folder,
        project_name: in_.project_name,
    })?;

    match docker_host.scheme {
        docker_host::Scheme::Ssh => deploy_remotely(&project, &docker_host, &in_.image_source_host),
        _ => deploy_locally::go(&project),
    }
}

pub struct In {
    pub compose_file: String,
    pub docker_host: Option<String>,
    pub image_source_host: Option<String>,
    pub project_folder: Option<String>,
    pub project_name: Option<String>,
}

fn deploy_remotely(
    project: &compose::Project,
    docker_host: &docker_host::Host,
    image_source_host: &Option<String>,
) -> anyhow::Result<()> {
    eprintln!("Assembling artifacts.");
    assemble_artifacts(project)?;
    eprintln!("Synchronizing artifacts.");
    synchronize_artifacts(project, docker_host)?;
    if let Some(image_source_host) = image_source_host {
        eprintln!("Transferring images.");
        transfer_images(project, docker_host, image_source_host)?;
    }
    eprintln!("Deploying on remote.");
    run_deploy_on_remote(project, docker_host)
}

fn assemble_artifacts(project: &compose::Project) -> anyhow::Result<()> {
    let local_workbench = path::Path::new(&project.x_wheelsticks.local_workbench);
    fs::create_dir_all(local_workbench)?;
    let contents = compose::print(project.clone()).context("Unable to print Compose file")?;
    let file = local_workbench.join("compose.yaml");
    fs::write(&file, contents).with_context(|| format!("Unable to write Compose file to {file:?}"))
}

fn synchronize_artifacts(
    project: &compose::Project,
    docker_host: &docker_host::Host,
) -> anyhow::Result<()> {
    let local_workbench = &project.x_wheelsticks.local_workbench;
    let source = format!("{local_workbench}/");

    let optional_user = docker_host
        .user
        .as_ref()
        .map(|user| format!("{user}@"))
        .unwrap_or_default();
    let host = &docker_host.hostname;
    let remote_workbench = &project.x_wheelsticks.remote_workbench;
    let destination = format!("{optional_user}{host}:{remote_workbench}");

    command::status_ok(
        process::Command::new("rsync")
            .args(["--archive", "--delete"])
            .args(
                docker_host
                    .port
                    .iter()
                    .flat_map(|port| ["--rsh".into(), format!("ssh -p {port:?}")]),
            )
            .arg("--")
            .args([source, destination]),
    )
}

fn transfer_images(
    project: &compose::Project,
    docker_host: &docker_host::Host,
    image_source_host: &str,
) -> anyhow::Result<()> {
    let project_name = &project.name;

    for service in project.services.keys() {
        let image_name = format!("{project_name}-{service}");

        let save_child = process::Command::new("docker")
            .args(["--host", image_source_host, "save", "--", &image_name])
            .stdout(process::Stdio::piped())
            .spawn()
            .with_context(|| format!("Unable to save image {image_name:?}"))?;

        let save_stdout = save_child
            .stdout
            .ok_or_else(|| anyhow::anyhow!("Unable to open stdout for image {image_name:?}"))?;

        command::status_ok(
            process::Command::new("docker")
                .args(["--host", &docker_host.url, "load"])
                .stdin(save_stdout),
        )
        .with_context(|| format!("Unable to load image {image_name:?}"))?;
    }

    Ok(())
}

fn run_deploy_on_remote(
    project: &compose::Project,
    docker_host: &docker_host::Host,
) -> anyhow::Result<()> {
    let optional_user = docker_host
        .user
        .as_ref()
        .map(|user| format!("{user}@"))
        .unwrap_or_default();
    let host = &docker_host.hostname;
    let optional_port = docker_host
        .port
        .map(|port| format!(":{port}"))
        .unwrap_or_default();
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
            .args(["--host", "unix:///tmp/todo.sock"]),
    )
}
