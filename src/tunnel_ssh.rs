use super::log;
use anyhow::Context;
use std::os::unix::process::CommandExt;
use std::process;

pub fn go(
    In {
        dry_run,
        local_port,
        remote_socket,
        ssh_config,
        ssh_host,
    }: In,
) -> anyhow::Result<()> {
    // TODO: Get socket path automatically, depending on container engine:
    // - `docker context inspect --format '{{.Endpoints.docker.Host}}'`
    // - `podman info --format '{{.Host.RemoteSocket.Path}}'`
    let remote_socket = remote_socket.unwrap_or("/run/user/1000/podman/podman.sock".into());

    let mut command = process::Command::new("ssh");
    command
        .args(ssh_config.iter().flat_map(|ssh_config| ["-F", ssh_config]))
        .args([
            "-f",
            "-L",
            &format!("localhost:{local_port}:{remote_socket}"),
            "-N",
            &ssh_host,
        ]);

    if dry_run {
        log::info!("Would execute: {command:?}");
        Ok(())
    } else {
        // Use `exec` so that SSH process is terminated on signals like TERM.
        Err(command.exec()).with_context(|| format!("Unable to execute: {command:?}"))?
    }
}

pub struct In {
    pub dry_run: bool,
    pub local_port: u16,
    pub remote_socket: Option<String>,
    pub ssh_config: Option<String>,
    pub ssh_host: String,
}
