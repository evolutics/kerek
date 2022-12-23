use anyhow::Context;
use std::process;

pub fn status(command: &mut process::Command) -> anyhow::Result<()> {
    match command.status() {
        Err(error) => Err(anyhow::anyhow!(error)),
        Ok(status) => {
            if status.success() {
                Ok(())
            } else {
                Err(anyhow::anyhow!("{status}"))
            }
        }
    }
    .with_context(|| format!("{command:?}"))
}
