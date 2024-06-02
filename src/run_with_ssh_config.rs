use super::command;
use super::log;
use anyhow::Context;
use std::env;
use std::fs;
use std::iter;
use std::os::unix::fs::PermissionsExt;
use std::path;
use std::process;

pub fn go(
    In {
        command,
        dry_run,
        ssh_config,
    }: In,
) -> anyhow::Result<()> {
    // TODO: Use `path::absolute` once stable.
    let ssh_config = ssh_config
        .canonicalize()
        .with_context(|| format!("Unable to make {ssh_config:?} absolute"))?;

    let real_ssh = command::stdout_utf8(process::Command::new("which").arg("ssh"))?;
    let real_ssh = real_ssh.trim_end();

    let custom_bin = tempfile::tempdir_in(".")?;
    let ssh_wrapper = custom_bin.path().join("ssh");

    log::debug!(
        "Wrapping {real_ssh:?} temporarily in {ssh_wrapper:?} for SSH config {ssh_config:?}."
    );
    fs::write(
        &ssh_wrapper,
        format!("#!/bin/sh\nexec {real_ssh:?} -F {ssh_config:?} \"$@\"\n"),
    )?;
    fs::set_permissions(ssh_wrapper, fs::Permissions::from_mode(0o755))?;

    let old_path = env::var_os("PATH").unwrap_or_default();
    let new_path = env::join_paths(
        iter::once(custom_bin.path().to_owned()).chain(env::split_paths(&old_path)),
    )?;

    let mut command_in_context = process::Command::new(&command[0]);
    command_in_context.args(&command[1..]).env("PATH", new_path);

    if dry_run {
        log::info!("Would run: {command_in_context:?}");
        Ok(())
    } else {
        command::status_ok(&mut command_in_context)
    }
}

pub struct In {
    pub command: Vec<String>,
    pub dry_run: bool,
    pub ssh_config: path::PathBuf,
}
