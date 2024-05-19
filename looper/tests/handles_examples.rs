use std::fs;
use std::io;
use std::path;
use std::process;

#[test]
fn compose() -> anyhow::Result<()> {
    test_one_offs("compose")
}

#[test]
fn log_only() -> anyhow::Result<()> {
    let folder = path::Path::new("examples/log_only");
    let log_file = folder.join("log.txt");
    fs::write(&log_file, "")?;

    assert!(!execute_subcommand("loop", folder)?.success());

    assert_eq!(
        fs::read_to_string(log_file)?,
        "Build
Deploy: staging
Env tests: staging
Deploy: production
Env tests: production
Move to next version
Build
Deploy: staging
Env tests: staging
Deploy: production
Env tests: production
Move to next version
Break
",
    );
    Ok(())
}

fn test_one_offs(example: &str) -> anyhow::Result<()> {
    let folder = ["examples", example].iter().collect::<path::PathBuf>();
    reset_fake_production(&folder)?;

    assert!(execute_subcommand("clean", &folder)?.success());
    assert!(execute_subcommand("run", &folder)?.success());
    assert!(execute_subcommand("dry-run", &folder)?.success());
    Ok(())
}

fn reset_fake_production(folder: &path::Path) -> anyhow::Result<()> {
    assert!(process::Command::new("vagrant")
        .arg("destroy")
        .arg("--force")
        .current_dir(folder)
        .status()?
        .success());
    assert!(process::Command::new("vagrant")
        .arg("up")
        .current_dir(folder)
        .status()?
        .success());
    let ssh_host = "production";
    let ssh_configuration = path::Path::new("safe/ssh_configuration");
    assert!(process::Command::new("vagrant")
        .arg("ssh-config")
        .arg("--host")
        .arg(ssh_host)
        .current_dir(folder)
        .stdout(fs::File::create(folder.join(ssh_configuration))?)
        .status()?
        .success());
    assert!(process::Command::new("wheelsticks")
        .arg("provision")
        .arg("--force")
        .arg("--ssh-config")
        .arg(ssh_configuration)
        .arg(ssh_host)
        .current_dir(folder)
        .status()?
        .success());
    Ok(())
}

fn execute_subcommand(subcommand: &str, folder: &path::Path) -> io::Result<process::ExitStatus> {
    process::Command::new(env!("CARGO_BIN_EXE_kerek"))
        .arg(subcommand)
        .current_dir(folder)
        .status()
}
