use std::env;
use std::fs;
use std::iter;
use std::path;
use std::process;

#[test]
fn go() -> anyhow::Result<()> {
    reset_vm()?;
    assert_command_in_context(process::Command::new("git").args([
        "clean",
        "--force",
        "-X",
        "--",
        ".wheelsticks",
    ]))?;

    assert_command_in_context(process::Command::new(EXECUTABLE).arg("render"))?;
    assert_command_in_context(process::Command::new(EXECUTABLE).args([
        "provision",
        "--deploy-user",
        DEPLOY_USER,
        "--host",
        &format!("ssh://{SSH_HOST}"),
    ]))?;
    assert_command_in_context(process::Command::new(EXECUTABLE).arg("build"))?;
    assert_command_in_context(process::Command::new(EXECUTABLE).args([
        "deploy",
        "--host",
        &format!("ssh://{DEPLOY_USER}@{SSH_HOST}"),
    ]))?;

    assert_command_in_context(process::Command::new("curl").args([
        "--fail",
        "--max-time",
        "3",
        "--retry",
        "99",
        "--retry-connrefused",
        "--retry-max-time",
        "15",
        "--show-error",
        &format!("http://{VM_IP_ADDRESS}"),
    ]))?;
    Ok(())
}

const DEPLOY_USER: &str = "wheelsticks";

const EXECUTABLE: &str = env!("CARGO_BIN_EXE_wheelsticks");

const FOLDER: &str = "example";

const SSH_CONFIGURATION_FILE: &str = "ssh_configuration";

const SSH_HOST: &str = "example";

const VM_IP_ADDRESS: &str = "192.168.60.97";

fn assert_command_in_context(command: &mut process::Command) -> anyhow::Result<()> {
    let original_path = env::var("PATH")?;
    let custom_path =
        env::join_paths(iter::once("custom_bin".into()).chain(env::split_paths(&original_path)))?;

    assert!(command
        .current_dir(FOLDER)
        .env("PATH", custom_path)
        .env("WHEELSTICKS_VM_IP_ADDRESS", VM_IP_ADDRESS)
        .status()?
        .success());
    Ok(())
}

fn reset_vm() -> anyhow::Result<()> {
    assert_command_in_context(process::Command::new("vagrant").args(["destroy", "--force"]))?;
    assert_command_in_context(process::Command::new("vagrant").arg("up"))?;
    assert_command_in_context(
        process::Command::new("vagrant")
            .args(["ssh-config", "--host", SSH_HOST])
            .stdout(fs::File::create(
                path::Path::new(FOLDER).join(SSH_CONFIGURATION_FILE),
            )?),
    )
}
