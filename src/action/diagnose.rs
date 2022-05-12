use crate::library::command;
use std::io;
use std::io::Write;
use std::process;

pub fn go() -> anyhow::Result<()> {
    command::status(process::Command::new("git").arg("version"))?;

    print!("kubectl ");
    io::stdout().flush()?;
    command::status(
        process::Command::new("kubectl")
            .arg("version")
            .arg("--client")
            .arg("--short"),
    )?;

    command::status(
        process::Command::new("skaffold")
            .arg("version")
            .arg("--output")
            .arg("Skaffold {{.Version}}\n"),
    )?;

    command::status(process::Command::new("ssh").arg("-V"))?;

    command::status(process::Command::new("vagrant").arg("--version"))
}
