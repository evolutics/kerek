use super::command;
use super::retry;
use std::fs;
use std::path;
use std::process;
use std::time;

pub fn go(in_: In) -> anyhow::Result<()> {
    do_scripted_provisioning(&in_)?;
    reboot(&in_)?;
    test_scripted_provisioning(&in_)
}

pub struct In<'a> {
    pub script_file: &'a path::Path,
    pub ssh_configuration_file: &'a path::Path,
    pub ssh_host: &'a str,
}

fn do_scripted_provisioning(in_: &In) -> anyhow::Result<()> {
    let script = fs::File::open(in_.script_file)?;
    command::status(
        process::Command::new("ssh")
            .arg("-F")
            .arg(in_.ssh_configuration_file)
            .arg(in_.ssh_host)
            .arg("--")
            .arg("bash")
            .arg("-s")
            .arg("--")
            .arg("do")
            .stdin(script),
    )
}

fn reboot(in_: &In) -> anyhow::Result<()> {
    command::status(
        process::Command::new("ssh")
            .arg("-F")
            .arg(in_.ssh_configuration_file)
            .arg("-f")
            .arg("-l")
            .arg("kerek")
            .arg(in_.ssh_host)
            .arg("--")
            .arg("sudo")
            .arg("reboot"),
    )
}

fn test_scripted_provisioning(in_: &In) -> anyhow::Result<()> {
    retry::go(retry::In {
        total_duration_limit: time::Duration::from_secs(150),
        retry_pause: time::Duration::from_secs(3),
        run: || {
            let script = fs::File::open(in_.script_file)?;
            command::status(
                process::Command::new("ssh")
                    .arg("-F")
                    .arg(in_.ssh_configuration_file)
                    .arg("-l")
                    .arg("kerek")
                    .arg(in_.ssh_host)
                    .arg("--")
                    .arg("bash")
                    .arg("-s")
                    .arg("--")
                    .arg("test")
                    .stdin(script),
            )
        },
    })
}
