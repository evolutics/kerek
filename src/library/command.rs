use anyhow::Context;
use std::io;
use std::process;

#[allow(dead_code)]
pub fn status_bit(command: &mut process::Command) -> anyhow::Result<bool> {
    go(command, process::Command::status, |status| {
        match status.code() {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => status_error(status),
        }
    })
}

#[allow(dead_code)]
pub fn status_ok(command: &mut process::Command) -> anyhow::Result<()> {
    go(command, process::Command::status, |status| {
        if status.success() {
            Ok(())
        } else {
            status_error(status)
        }
    })
}

#[allow(dead_code)]
pub fn stderr_utf8(command: &mut process::Command) -> anyhow::Result<String> {
    go(command, process::Command::output, |output| {
        if output.status.success() {
            String::from_utf8(output.stderr).context("Stderr is not valid UTF-8")
        } else {
            status_error(output.status)
        }
    })
}

#[allow(dead_code)]
pub fn stdout_utf8(command: &mut process::Command) -> anyhow::Result<String> {
    go(command, process::Command::output, |output| {
        if output.status.success() {
            String::from_utf8(output.stdout).context("Stdout is not valid UTF-8")
        } else {
            status_error(output.status)
        }
    })
}

fn go<
    F: FnOnce(&mut process::Command) -> io::Result<T>,
    G: FnOnce(T) -> anyhow::Result<U>,
    T,
    U,
>(
    command: &mut process::Command,
    run: F,
    evaluate: G,
) -> anyhow::Result<U> {
    match run(command) {
        Err(error) => Err(anyhow::anyhow!(error)),
        Ok(value) => evaluate(value),
    }
    .with_context(|| format!("Unable to run command: {command:?}"))
}

fn status_error<T>(status: process::ExitStatus) -> anyhow::Result<T> {
    Err(anyhow::anyhow!("{status}"))
}
