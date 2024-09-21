use anyhow::Context;
use serde::de;
use std::io::Write;
use std::process;
use std::thread;

pub fn piped_ok(commands: &mut [&mut process::Command]) -> anyhow::Result<()> {
    let mut children = Vec::<process::Child>::new();

    let mut pipeline = commands.iter_mut().peekable();
    while let Some(command) = pipeline.next() {
        (|| {
            if let Some(previous) = children.last_mut() {
                let stdout = previous.stdout.take().context("Unable to open stdout")?;
                command.stdin(stdout);
            }
            if pipeline.peek().is_some() {
                command.stdout(process::Stdio::piped());
            }

            let child = command.spawn().context("Unable to spawn")?;
            children.push(child);

            Ok(())
        })()
        .command_context(command)?;
    }

    for (index, child) in children.iter_mut().enumerate() {
        let command = &commands[index];
        (|| status_result(child.wait()?))().command_context(command)?;
    }

    Ok(())
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

    #[test_case::test_case(vec![], true; "success 0")]
    #[test_case::test_case(vec![invalid_program_()], false; "invalid 0/1")]
    #[test_case::test_case(vec![bash("false")], false; "failure 0/1")]
    #[test_case::test_case(vec![bash("true")], true; "success 1")]
    #[test_case::test_case(vec![invalid_program_(), bash("true")], false; "invalid 0/2")]
    #[test_case::test_case(vec![bash("true"), invalid_program_()], false; "invalid 1/2")]
    #[test_case::test_case(vec![bash("false"), bash("true")], false; "failure 0/2")]
    #[test_case::test_case(vec![bash("true"), bash("false")], false; "failure 1/2")]
    #[test_case::test_case(vec![bash("echo 'Hi'"), bash("[[ $(cat) == 'Hi' ]]")], true; "success 2")]
    #[test_case::test_case(vec![invalid_program_(), bash("true"), bash("true")], false; "invalid 0/3")]
    #[test_case::test_case(vec![bash("true"), invalid_program_(), bash("true")], false; "invalid 1/3")]
    #[test_case::test_case(vec![bash("true"), bash("true"), invalid_program_()], false; "invalid 2/3")]
    #[test_case::test_case(vec![bash("false"), bash("true"), bash("true")], false; "failure 0/3")]
    #[test_case::test_case(vec![bash("true"), bash("false"), bash("true")], false; "failure 1/3")]
    #[test_case::test_case(vec![bash("true"), bash("true"), bash("false")], false; "failure 2/3")]
    #[test_case::test_case(vec![bash("echo 'Hi'"), bash("rev"), bash("[[ $(cat) == 'iH' ]]")], true; "success 3")]
    fn piped_ok_handles(mut commands: Vec<process::Command>, expected: bool) {
        let mut commands = commands.iter_mut().collect::<Vec<_>>();
        assert_eq!(piped_ok(commands.as_mut_slice()).is_ok(), expected)
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
