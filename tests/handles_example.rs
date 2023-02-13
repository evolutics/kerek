use std::fs;
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

    assert_command_in_context(process::Command::new(EXECUTABLE).args([
        "provision",
        "--deploy-user",
        DEPLOY_USER,
        "--ssh-configuration",
        SSH_CONFIGURATION_FILE,
        "--",
        SSH_HOST,
    ]))?;
    assert_command_in_context(process::Command::new(EXECUTABLE).arg("build"))?;
    assert_command_in_context(process::Command::new(EXECUTABLE).args([
        "deploy",
        "--ssh-configuration",
        SSH_CONFIGURATION_FILE,
        "--ssh-user",
        DEPLOY_USER,
        "--",
        SSH_HOST,
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
    assert!(command.current_dir(FOLDER).status()?.success());
    Ok(())
}

fn reset_vm() -> anyhow::Result<()> {
    assert_command_in_context(
        process::Command::new("vagrant")
            .args(["destroy", "--force"])
            .env("VM_IP_ADDRESS", VM_IP_ADDRESS),
    )?;
    assert_command_in_context(
        process::Command::new("vagrant")
            .arg("up")
            .env("VM_IP_ADDRESS", VM_IP_ADDRESS),
    )?;
    assert_command_in_context(
        process::Command::new("vagrant")
            .args(["ssh-config", "--host", SSH_HOST])
            .env("VM_IP_ADDRESS", VM_IP_ADDRESS)
            .stdout(fs::File::create(
                path::Path::new(FOLDER).join(SSH_CONFIGURATION_FILE),
            )?),
    )
}
