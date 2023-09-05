use anyhow::Context;
use serde::de;
use std::io;
use std::process;

pub fn status_ok(command: &mut process::Command) -> anyhow::Result<()> {
    go(command, process::Command::status, |status| {
        if status.success() {
            Ok(())
        } else {
            status_error(status)
        }
    })
}

pub fn stdout_json<T: de::DeserializeOwned>(command: &mut process::Command) -> anyhow::Result<T> {
    go(command, process::Command::output, |output| {
        if output.status.success() {
            serde_json::from_slice(&output.stdout).context("Unable to deserialize JSON from stdout")
        } else {
            status_error(output.status)
        }
    })
}

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
    .with_context(|| format!("Unable to run command {command:?}"))
}

fn status_error<T>(status: process::ExitStatus) -> anyhow::Result<T> {
    Err(anyhow::anyhow!("{status}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test_case::test_case(invalid_program_(), false; "invalid program")]
    #[test_case::test_case(shell("exit 0"), true; "success")]
    #[test_case::test_case(shell("exit 1"), false; "failure")]
    fn status_ok_handles(mut command: process::Command, expected: bool) {
        assert_eq!(status_ok(&mut command).is_ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(shell("exit 1"), None; "failure")]
    #[test_case::test_case(shell("printf '\"Hi\"'"), Some("Hi".into()); "success")]
    fn stdout_json_handles(mut command: process::Command, expected: Option<String>) {
        assert_eq!(stdout_json(&mut command).ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(shell("exit 1"), None; "failure")]
    #[test_case::test_case(shell("printf Hi"), Some("Hi".into()); "success")]
    fn stdout_utf8_handles(mut command: process::Command, expected: Option<String>) {
        assert_eq!(stdout_utf8(&mut command).ok(), expected)
    }

    fn invalid_program_() -> process::Command {
        process::Command::new("")
    }

    fn shell(script: &str) -> process::Command {
        let mut command = process::Command::new("sh");
        command.args(["-c", script]);
        command
    }
}
