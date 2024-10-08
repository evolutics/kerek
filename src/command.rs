use super::log;
use anyhow::Context;
use serde::de;
use std::io;
use std::io::Write;
use std::process;
use std::thread;

pub fn piped_ok<'a, T: IntoIterator<Item = &'a mut process::Command>>(
    commands: T,
) -> anyhow::Result<()> {
    let mut commands = commands.into_iter().collect::<Vec<_>>();

    match commands.split_last_mut() {
        None => Ok(()),
        Some((last_command, first_commands)) => {
            let mut stdio = None;
            let mut processes = vec![];

            for command in first_commands {
                if let Some(stdin) = stdio {
                    command.stdin(stdin);
                }
                command.stdout(process::Stdio::piped());

                let mut process = Process {
                    child: command.spawn().command_context(command)?,
                    command,
                };

                stdio = Some(
                    process
                        .child
                        .stdout
                        .take()
                        .context("Unable to open stdout")
                        .command_context(command)?,
                );

                processes.push(process);
            }

            if let Some(stdin) = stdio {
                last_command.stdin(stdin);
            }
            let last_status = last_command.status().command_context(last_command)?;

            // Status of whole pipeline is status of its last command. However,
            // for better user-facing error messages, we return first instead of
            // last error because root cause is usually with first error.

            if !last_status.success() {
                for Process {
                    ref mut child,
                    command,
                } in processes
                {
                    if let Some(status) = child.try_wait().command_context(command)? {
                        status_result(status).command_context(command)?;
                    }
                }
            }

            status_result(last_status).command_context(last_command)
        }
    }
}

pub fn status_ok(command: &mut process::Command) -> anyhow::Result<()> {
    (|| status_result(command.status()?))().command_context(command)
}

pub fn stdin_ok(input: &'static [u8], command: &mut process::Command) -> anyhow::Result<()> {
    (|| {
        let mut child = command.stdin(process::Stdio::piped()).spawn()?;
        let mut stdin = child.stdin.take().context("Unable to open stdin")?;
        thread::spawn(move || stdin.write_all(input).context("Unable to write to stdin"));
        status_result(child.wait().context("Unable to wait")?)
    })()
    .command_context(command)
}

pub fn stdout_json<T: de::DeserializeOwned>(command: &mut process::Command) -> anyhow::Result<T> {
    (|| {
        let output = command.stderr(process::Stdio::inherit()).output()?;
        status_result(output.status)?;
        serde_json::from_slice(&output.stdout).context("Unable to deserialize JSON from stdout")
    })()
    .command_context(command)
}

pub fn stdout_table<const N: usize>(
    command: &mut process::Command,
) -> anyhow::Result<Vec<[String; N]>> {
    (|| {
        let output = command.stderr(process::Stdio::inherit()).output()?;
        status_result(output.status)?;

        let table = String::from_utf8(output.stdout).context("Stdout is not valid UTF-8")?;
        table
            .lines()
            .enumerate()
            .map(|(row_index, row)| {
                let fields = row
                    .split_whitespace()
                    .map(|field| field.into())
                    .collect::<Vec<_>>();

                fields.try_into().map_err(|fields: Vec<_>| {
                    let line_number = row_index + 1;
                    let field_count = fields.len();
                    anyhow::anyhow!(
                        "Unable to parse result line {line_number}, \
                            expected {N} fields \
                            but got {field_count}: {row:?}"
                    )
                })
            })
            .collect::<anyhow::Result<_>>()
    })()
    .command_context(command)
}

pub fn stdout_utf8(command: &mut process::Command) -> anyhow::Result<String> {
    (|| {
        let output = command.stderr(process::Stdio::inherit()).output()?;
        status_result(output.status)?;
        String::from_utf8(output.stdout).context("Stdout is not valid UTF-8")
    })()
    .command_context(command)
}

trait CommandContext<T> {
    fn command_context(self, command: &process::Command) -> anyhow::Result<T>;
}

impl<T> CommandContext<T> for anyhow::Result<T> {
    fn command_context(self, command: &process::Command) -> anyhow::Result<T> {
        self.with_context(|| format!("Error with command: {command:?}"))
    }
}

impl<T> CommandContext<T> for io::Result<T> {
    fn command_context(self, command: &process::Command) -> anyhow::Result<T> {
        self.with_context(|| format!("Error with command: {command:?}"))
    }
}

struct Process<'a> {
    child: process::Child,
    command: &'a process::Command,
}

impl Drop for Process<'_> {
    fn drop(&mut self) {
        let Self { child, command } = self;

        let has_process_exited = child.try_wait().is_ok_and(|status| status.is_some());

        if !has_process_exited {
            let pid = child.id();
            log::debug!("Killing process {pid} from command: {command:?}");
            if let Err(error) = child.kill() {
                log::error!("Error killing process {pid}: {error}");
            }
        }
    }
}

fn status_result(status: process::ExitStatus) -> anyhow::Result<()> {
    if status.success() {
        Ok(())
    } else {
        Err(anyhow::anyhow!("{status}"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test_case::test_case(&[], true; "0")]
    #[test_case::test_case(&[""], false; "invalid 1")]
    #[test_case::test_case(&["false"], false; "failure 1")]
    #[test_case::test_case(&["true"], true; "success 1")]
    #[test_case::test_case(&["", "true"], false; "invalid 0/2")]
    #[test_case::test_case(&["true", ""], false; "invalid 1/2")]
    #[test_case::test_case(&["false", "true"], true; "false, success 2")]
    #[test_case::test_case(&["true", "false"], false; "true, failure 2")]
    #[test_case::test_case(&["echo 'Hi'", "[[ $(cat) == 'Hi' ]]"], true; "pipe 2")]
    #[test_case::test_case(&["yes", "false"], false; "loop, failure 2")]
    #[test_case::test_case(&["yes", "true"], true; "loop, success 2")]
    #[test_case::test_case(&["", "true", "true"], false; "invalid 0/3")]
    #[test_case::test_case(&["true", "", "true"], false; "invalid 1/3")]
    #[test_case::test_case(&["true", "true", ""], false; "invalid 2/3")]
    #[test_case::test_case(&["false", "false", "true"], true; "false, success 3")]
    #[test_case::test_case(&["true", "true", "false"], false; "true, failure 3")]
    #[test_case::test_case(&["echo 'Hi'", "rev", "[[ $(cat) == 'iH' ]]"], true; "pipe 3")]
    #[test_case::test_case(&["yes", "yes", "false"], false; "loop, failure 3")]
    #[test_case::test_case(&["yes", "yes", "true"], true; "loop, success 3")]
    fn piped_ok_handles(commands: &[&str], expected: bool) {
        let mut commands = commands
            .iter()
            .map(|command| {
                if command.is_empty() {
                    invalid_program_()
                } else {
                    bash(command)
                }
            })
            .collect::<Vec<_>>();
        assert_eq!(piped_ok(commands.iter_mut()).is_ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), false; "invalid program")]
    #[test_case::test_case(bash("true"), true; "success")]
    #[test_case::test_case(bash("false"), false; "failure")]
    fn status_ok_handles(mut command: process::Command, expected: bool) {
        assert_eq!(status_ok(&mut command).is_ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), false; "invalid program")]
    #[test_case::test_case(bash("[[ $(cat) == 'Hi' ]]"), true; "success")]
    #[test_case::test_case(bash("[[ $(cat) != 'Hi' ]]"), false; "failure")]
    fn stdin_ok_handles(mut command: process::Command, expected: bool) {
        assert_eq!(stdin_ok("Hi".as_bytes(), &mut command).is_ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(bash("false"), None; "failure")]
    #[test_case::test_case(bash("echo '\"Hi\"'"), Some("Hi".into()); "success")]
    fn stdout_json_handles(mut command: process::Command, expected: Option<String>) {
        assert_eq!(stdout_json(&mut command).ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(bash("false"), None; "failure")]
    #[test_case::test_case(
        bash("printf '13 a  b\n 8 x\tyz'"),
        Some(vec![
            ["13".into(), "a".into(), "b".into()],
            ["8".into(), "x".into(), "yz".into()],
        ]);
        "success"
    )]
    fn stdout_table_handles(mut command: process::Command, expected: Option<Vec<[String; 3]>>) {
        assert_eq!(stdout_table(&mut command).ok(), expected)
    }

    #[test_case::test_case(invalid_program_(), None; "invalid program")]
    #[test_case::test_case(bash("false"), None; "failure")]
    #[test_case::test_case(bash("printf 'Hi'"), Some("Hi".into()); "success")]
    fn stdout_utf8_handles(mut command: process::Command, expected: Option<String>) {
        assert_eq!(stdout_utf8(&mut command).ok(), expected)
    }

    fn invalid_program_() -> process::Command {
        process::Command::new("")
    }

    fn bash(script: &str) -> process::Command {
        let mut command = process::Command::new("bash");
        command.args(["-c", script]);
        command
    }
}
