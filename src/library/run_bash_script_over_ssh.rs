use super::run_command;
use std::fs;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let script = fs::File::open(in_.script_file)?;
    run_command::go(
        process::Command::new("ssh")
            .arg("-F")
            .arg(in_.configuration_file)
            .arg(in_.host)
            .arg("bash")
            .stdin(script),
    )
}

pub struct In<'a> {
    pub configuration_file: &'a path::Path,
    pub host: &'a str,
    pub script_file: &'a path::Path,
}
