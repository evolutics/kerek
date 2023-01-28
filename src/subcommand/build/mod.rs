use crate::library::command;
use anyhow::Context;
use std::fs;
use std::process;

pub fn go() -> anyhow::Result<()> {
    let build = tempfile::NamedTempFile::new()?;
    fs::write(&build, include_str!("build.py")).context("Unable to write file: build.py")?;

    command::status_ok(
        process::Command::new("python3")
            .arg("--")
            .arg(build.as_ref()),
    )
}
