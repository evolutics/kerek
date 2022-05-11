use crate::library::run_command;
use std::io;
use std::io::Write;
use std::process;

pub fn go() -> anyhow::Result<()> {
    run_command::go(process::Command::new("git").arg("version"))?;

    print!("kubectl ");
    io::stdout().flush()?;
    run_command::go(
        process::Command::new("kubectl")
            .arg("version")
            .arg("--client")
            .arg("--short"),
    )?;

    run_command::go(
        process::Command::new("skaffold")
            .arg("version")
            .arg("--output")
            .arg("Skaffold {{.Version}}\n"),
    )?;

    run_command::go(process::Command::new("ssh").arg("-V"))?;

    run_command::go(process::Command::new("vagrant").arg("--version"))
}
