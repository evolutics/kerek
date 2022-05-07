use crate::library::run_command;
use std::process;

pub fn go() -> Result<(), String> {
    run_command::go(process::Command::new("git").arg("version"))?;

    run_command::go(process::Command::new("skaffold").args([
        "version",
        "--output",
        "Skaffold {{.Version}}\n",
    ]))?;

    run_command::go(process::Command::new("vagrant").arg("--version"))
}