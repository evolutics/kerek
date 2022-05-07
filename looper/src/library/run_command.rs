use anyhow::Context;
use std::process;

pub fn go(command: &mut process::Command) -> anyhow::Result<()> {
    run_raw(command).with_context(|| format!("{command:?}"))
}

fn run_raw(command: &mut process::Command) -> anyhow::Result<()> {
    let status = command.status()?;
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{status}"))
    }
}
