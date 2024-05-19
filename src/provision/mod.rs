use super::command;
use std::process;

pub fn go(
    In {
        ssh_config,
        ssh_host,
    }: In,
) -> anyhow::Result<()> {
    command::stdin_ok(
        include_bytes!("provision_on_remote.sh"),
        process::Command::new("ssh")
            .args(ssh_config.iter().flat_map(|ssh_config| ["-F", ssh_config]))
            .arg(ssh_host),
    )
}

pub struct In {
    pub ssh_config: Option<String>,
    pub ssh_host: String,
}
