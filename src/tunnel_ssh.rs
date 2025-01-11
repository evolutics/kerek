use super::command;
use super::log;
use super::ssh;
use anyhow::Context;
use std::path;
use std::process;

pub fn go(
    In {
        container_engine,
        dry_run,
        local_socket,
        remote_socket,
        ssh_cli,
        ssh_host,
    }: In,
) -> anyhow::Result<()> {
    let local_socket = path::absolute(&local_socket)
        .with_context(|| format!("Unable to make {local_socket:?} absolute"))?;
    let remote_socket = match remote_socket {
        None => infer_remote_socket(RemoteConfig {
            container_engine: &container_engine,
            ssh_cli: &ssh_cli,
            ssh_host: &ssh_host,
        })
        .context("Unable to infer remote socket")?,
        Some(remote_socket) => remote_socket,
    };

    let mut command = ssh_cli.command();
    command.args([
        "-f",
        "-N",
        "-o",
        &format!("LocalForward {local_socket:?} {remote_socket:?}"),
        "-o",
        "StreamLocalBindUnlink=yes", // Required to reuse socket file.
        &ssh_host,
    ]);

    if dry_run {
        log::info!("Would execute: {command:?}");
        Ok(())
    } else {
        // Use `exec` so that SSH tunnel is ready when main process returns.
        exec(command)
    }
}

pub struct In<'a> {
    pub container_engine: String,
    pub dry_run: bool,
    pub local_socket: String,
    pub remote_socket: Option<String>,
    pub ssh_cli: ssh::Cli<'a>,
    pub ssh_host: String,
}

struct RemoteConfig<'a> {
    container_engine: &'a str,
    ssh_cli: &'a ssh::Cli<'a>,
    ssh_host: &'a str,
}

fn infer_remote_socket(
    RemoteConfig {
        container_engine,
        ssh_cli,
        ssh_host,
    }: RemoteConfig,
) -> anyhow::Result<String> {
    let mut command = ssh_cli.command();
    command.arg(ssh_host);

    match container_engine {
        "docker" => command.args([
            "docker",
            "context",
            "inspect",
            "--format",
            "{{.Endpoints.docker.Host}}",
        ]),
        "podman" => command.args(["podman", "info", "--format", "{{.Host.RemoteSocket.Path}}"]),
        _ => command.args(["echo", "${DOCKER_HOST}"]),
    };

    let socket = command::stdout_utf8(&mut command)?;
    let socket = socket.trim_end();
    let socket = socket.strip_prefix("unix://").unwrap_or(socket);

    log::info!("Inferred remote socket: {socket:?}");
    Ok(socket.into())
}

#[cfg(not(unix))]
fn exec(mut _command: process::Command) -> anyhow::Result<()> {
    Err(anyhow::anyhow!("`exec` not supported"))
}

#[cfg(unix)]
fn exec(mut command: process::Command) -> anyhow::Result<()> {
    use std::os::unix::process::CommandExt;
    Err(command.exec()).with_context(|| format!("Unable to execute: {command:?}"))?
}
