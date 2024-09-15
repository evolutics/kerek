use super::command;
use super::log;
use super::ssh;
use std::io;
use std::io::Write;
use std::process;

pub fn go(
    In {
        container_engine,
        dry_run,
        force,
        has_ssh_config_override,
        host,
        ssh_cli,
    }: In,
) -> anyhow::Result<()> {
    if !force {
        confirm_with_user(&format!(
            "About to provision {host:?}, making system-wide changes! \
            Are you sure?",
        ))?;
    }

    if dry_run {
        log::info!("Would provision host {host:?}.");
        Ok(())
    } else {
        let mut command = if host == "localhost" && !has_ssh_config_override {
            let mut command = process::Command::new("bash");
            command.env("CONTAINER_ENGINE", container_engine);
            command
        } else {
            let mut command = ssh_cli.command();
            command.args([
                &host,
                &format!("CONTAINER_ENGINE={container_engine}"),
                "bash",
            ]);
            command
        };

        command::stdin_ok(include_bytes!("provision_on_host.sh"), &mut command)
    }
}

pub struct In<'a> {
    pub container_engine: String,
    pub dry_run: bool,
    pub force: bool,
    pub has_ssh_config_override: bool,
    pub host: String,
    pub ssh_cli: ssh::Cli<'a>,
}

fn confirm_with_user(question: &str) -> anyhow::Result<()> {
    let yes = "yes";

    print!("{question} Continuing only on {yes:?}. ");
    io::stdout().flush()?;

    let mut answer = String::new();
    io::stdin().read_line(&mut answer)?;

    if answer.trim_end() == yes {
        Ok(())
    } else {
        Err(anyhow::anyhow!("Aborted by user"))
    }
}
