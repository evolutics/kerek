use anyhow::Context;
use serde::de;
use std::io;
use std::process;
use std::time;
use wait_timeout::ChildExt;

pub fn status_bit(command: &mut process::Command) -> anyhow::Result<bool> {
    go(command, process::Command::status, |status| {
        match status.code() {
            Some(0) => Ok(false),
            Some(1) => Ok(true),
            _ => status_error(status),
        }
    })
}

pub fn status_ok(command: &mut process::Command) -> anyhow::Result<()> {
    go(command, process::Command::status, |status| {
        if status.success() {
            Ok(())
        } else {
            status_error(status)
        }
    })
}

pub fn status_within_time(
    command: &mut process::Command,
    timeout: time::Duration,
) -> anyhow::Result<StatusWithinTime> {
    go(
        command,
        |command| {
            let mut child = command.spawn()?;
            let status = child.wait_timeout(timeout)?;
            Ok((child, status))
        },
        |(mut child, status)| {
            Ok(match status {
                None => {
                    let _ = child.kill();
                    let _ = child.wait();
                    StatusWithinTime::Timeout
                }
                Some(status) => {
                    if status.success() {
                        StatusWithinTime::Success
                    } else {
                        StatusWithinTime::Failure
                    }
                }
            })
        },
    )
}

#[derive(Debug, PartialEq)]
pub enum StatusWithinTime {
    Failure,
    Success,
    Timeout,
}

#[allow(dead_code)]
pub fn stdout_json<T: de::DeserializeOwned>(command: &mut process::Command) -> anyhow::Result<T> {
    go(command, process::Command::output, |output| {
        if output.status.success() {
            serde_json::from_slice(&output.stdout).context("Unable to deserialize JSON from stdout")
        } else {
            status_error(output.status)
        }
    })
}

#[allow(dead_code)]
pub fn stdout_jsons<T: de::DeserializeOwned>(
    command: &mut process::Command,
) -> anyhow::Result<Vec<T>> {
    go(command, process::Command::output, |output| {
        if output.status.success() {
            let stream = serde_json::Deserializer::from_slice(&output.stdout).into_iter();
            let mut values = vec![];
            for (index, value) in stream.enumerate() {
                let value = value
                    .with_context(|| format!("Unable to deserialize JSON #{index} from stdout"))?;
                values.push(value);
            }
            Ok(values)
        } else {
            status_error(output.status)
        }
    })
}

pub fn stdout_raw(command: &mut process::Command) -> anyhow::Result<Vec<u8>> {
    go(command, process::Command::output, |output| {
        if output.status.success() {
            Ok(output.stdout)
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

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(shell("exit 0"), Some(false); "zero")]
    #[test_case::test_case(shell("exit 1"), Some(true); "one")]
    #[test_case::test_case(shell("exit 2"), None; "other")]
    fn status_bit_handles(mut command: process::Command, expected: Option<bool>) {
        assert_eq!(status_bit(&mut command).ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), false; "invalid program")]
    #[test_case::test_case(shell("exit 0"), true; "success")]
    #[test_case::test_case(shell("exit 1"), false; "failure")]
    fn status_ok_handles(mut command: process::Command, expected: bool) {
        assert_eq!(status_ok(&mut command).is_ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(shell("exit 0"), Some(StatusWithinTime::Success); "success")]
    #[test_case::test_case(shell("exit 1"), Some(StatusWithinTime::Failure); "failure")]
    #[test_case::test_case(shell("sleep 5"), Some(StatusWithinTime::Timeout); "timeout")]
    fn status_within_time_handles(
        mut command: process::Command,
        expected: Option<StatusWithinTime>,
    ) {
        assert_eq!(
            status_within_time(&mut command, time::Duration::from_secs_f32(0.01)).ok(),
            expected,
        )
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(shell("exit 1"), None; "failure")]
    #[test_case::test_case(shell("printf '\"Hi\"'"), Some("Hi".into()); "success")]
    fn stdout_json_handles(mut command: process::Command, expected: Option<String>) {
        assert_eq!(stdout_json(&mut command).ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(shell("exit 1"), None; "failure")]
    #[test_case::test_case(shell("printf '3 5'"), Some(vec![3, 5]); "success")]
    fn stdout_jsons_handles(mut command: process::Command, expected: Option<Vec<i8>>) {
        assert_eq!(stdout_jsons(&mut command).ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(shell("exit 1"), None; "failure")]
    #[test_case::test_case(shell("printf Hi"), Some("Hi".into()); "success")]
    fn stdout_raw_handles(mut command: process::Command, expected: Option<Vec<u8>>) {
        assert_eq!(stdout_raw(&mut command).ok(), expected)
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
