use super::command;
use super::log;
use std::io;
use std::io::Write;
use std::process;

pub fn go(
    In {
        force,
        host,
        ssh_config,
    }: In,
) -> anyhow::Result<()> {
    if !force {
        confirm_with_user(&format!("About to provision {host:?}! Are you sure?"))?;
    }

    let host = match host.split_once("://") {
        None if host == "localhost" => Host::Localhost,
        Some(("ssh", host)) => Host::Ssh { host },
        Some(("vagrant", vm)) => Host::Vagrant {
            vm: (!vm.is_empty()).then_some(vm),
        },
        _ => Host::Ssh { host: &host },
    };

    let mut command = match host {
        Host::Localhost => process::Command::new("bash"),
        Host::Ssh { host } => {
            let mut command = process::Command::new("ssh");
            command
                .args(ssh_config.iter().flat_map(|ssh_config| ["-F", ssh_config]))
                .args([host, "bash"]);
            command
        }
        Host::Vagrant { vm } => {
            command::status_ok(vagrant().arg("up").args(vm))?;
            let mut command = vagrant();
            command.args(["ssh", "--command", "bash"]).args(vm).args(
                ssh_config
                    .iter()
                    .flat_map(|ssh_config| ["--", "-F", ssh_config]),
            );
            command
        }
    };

    command::stdin_ok(include_bytes!("provision_on_host.sh"), &mut command)
}

pub struct In {
    pub force: bool,
    pub host: String,
    pub ssh_config: Option<String>,
}

enum Host<'a> {
    Localhost,
    Ssh { host: &'a str },
    Vagrant { vm: Option<&'a str> },
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

fn vagrant() -> process::Command {
    let mut command = process::Command::new("vagrant");
    command.args(
        (log::level() <= log::Level::Debug)
            .then_some("--debug")
            .iter(),
    );
    command
}
