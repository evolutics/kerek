use super::run_command;
use std::fs;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let script = fs::File::open(in_.script_file)?;
    run_command::go(
        process::Command::new("ssh")
            .args(["-F", in_.configuration_file, in_.hostname, "bash"])
            .stdin(script),
    )
}

pub struct In<'a> {
    pub configuration_file: &'a str,
    pub hostname: &'a str,
    pub script_file: &'a str,
}
