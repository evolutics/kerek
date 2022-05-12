use anyhow::Context;
use std::io;
use std::process;

pub fn status(command: &mut process::Command) -> anyhow::Result<()> {
    go(command, process::Command::status, |status| *status)?;
    Ok(())
}

#[allow(dead_code)]
pub fn stderr_utf8(command: &mut process::Command) -> anyhow::Result<String> {
    output_utf8(command.stdout(process::Stdio::inherit()), |output| {
        output.stderr
    })
}

#[allow(dead_code)]
pub fn stdout_utf8(command: &mut process::Command) -> anyhow::Result<String> {
    output_utf8(command.stderr(process::Stdio::inherit()), |output| {
        output.stdout
    })
}

fn go<F: Fn(&mut process::Command) -> io::Result<T>, G: Fn(&T) -> process::ExitStatus, T>(
    command: &mut process::Command,
    run: F,
    get_status: G,
) -> anyhow::Result<T> {
    match run(command) {
        Err(error) => Err(anyhow::anyhow!(error)),
        Ok(done_command) => {
            let status = get_status(&done_command);
            if status.success() {
                Ok(done_command)
            } else {
                Err(anyhow::anyhow!("{status}"))
            }
        }
    }
    .with_context(|| format!("{command:?}"))
}

fn output_utf8<F: Fn(process::Output) -> Vec<u8>>(
    command: &mut process::Command,
    get_raw: F,
) -> anyhow::Result<String> {
    let output = go(command, process::Command::output, |output| output.status)?;
    let raw = get_raw(output);
    String::from_utf8(raw).with_context(|| "Not valid UTF-8")
}
