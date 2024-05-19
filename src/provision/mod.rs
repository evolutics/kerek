use super::command;
use std::borrow;
use std::io;
use std::io::Write;
use std::process;

pub fn go(
    In {
        force,
        ssh_config,
        ssh_host,
    }: In,
) -> anyhow::Result<()> {
    if !force {
        let host = match &ssh_host {
            None => borrow::Cow::from("your localhost"),
            Some(ssh_host) => format!("host {ssh_host:?}").into(),
        };
        confirm_with_user(&format!("About to provision {host}! Are you sure?"))?;
    }

    let mut command = match ssh_host {
        None => process::Command::new("bash"),
        Some(ssh_host) => {
            let mut command = process::Command::new("ssh");
            command
                .args(ssh_config.iter().flat_map(|ssh_config| ["-F", ssh_config]))
                .arg(ssh_host);
            command
        }
    };

    command::stdin_ok(include_bytes!("provision_on_host.sh"), &mut command)
}

pub struct In {
    pub force: bool,
    pub ssh_config: Option<String>,
    pub ssh_host: Option<String>,
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
