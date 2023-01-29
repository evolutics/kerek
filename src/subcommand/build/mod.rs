use crate::library::command;
use anyhow::Context;
use std::fs;
use std::path;
use std::process;

pub fn go(_configuration: path::PathBuf) -> anyhow::Result<()> {
    let build = tempfile::NamedTempFile::new()?;
    fs::write(&build, include_str!("build.py")).context("Unable to write file: build.py")?;

    command::status_ok(
        process::Command::new("python3")
            .arg("--")
            .arg(build.as_ref()),
    )
}
