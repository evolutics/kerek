use crate::library::command;
use crate::library::configuration;
use anyhow::Context;
use std::fs;
use std::path;
use std::process;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(&configuration)?;

    let build = tempfile::NamedTempFile::new()?;
    fs::write(&build, include_str!("build.py")).context("Unable to write file: build.py")?;

    // TODO: Short-circuit if building to deploy on same machine without SSH.

    command::status_ok(
        process::Command::new("python3")
            .arg("--")
            .arg(build.as_ref())
            .env(
                "WHEELSTICKS_BUILD_CONTEXTS",
                configuration.x_wheelsticks.build_contexts.join(":"),
            )
            .env(
                "WHEELSTICKS_WORKBENCH",
                configuration.x_wheelsticks.workbench,
            ),
    )
}
