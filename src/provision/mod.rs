use super::command;
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
        confirm_with_user(&format!("About to provision {ssh_host:?}! Are you sure?"))?;
    }

    command::stdin_ok(
        include_bytes!("provision_on_remote.sh"),
        process::Command::new("ssh")
            .args(ssh_config.iter().flat_map(|ssh_config| ["-F", ssh_config]))
            .arg(ssh_host),
    )
}

pub struct In {
    pub force: bool,
    pub ssh_config: Option<String>,
    pub ssh_host: String,
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
