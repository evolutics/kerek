use std::fs;
use std::io;
use std::path;
use std::process;

#[test]
fn default() -> anyhow::Result<()> {
    test("default")
}

#[test]
fn kubernetes() -> anyhow::Result<()> {
    test("kubernetes")
}

fn test(example: &str) -> anyhow::Result<()> {
    let folder = ["examples", example].iter().collect::<path::PathBuf>();
    reset_fake_production(&folder)?;
    let log_file = folder.join("log.txt");
    fs::write(&log_file, "")?;

    assert!(clean(&folder)?.success());
    assert!(provision(&folder)?.success());
    assert!(!run(&folder)?.success());

    assert_eq!(
        fs::read_to_string(log_file)?,
        "Base tests
Smoke tests: staging
Acceptance tests
Smoke tests: production
Move to next version
Base tests
",
    );

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
    let ssh_configuration = fs::File::create(folder.join("safe/ssh_configuration"))?;
    assert!(process::Command::new("vagrant")
        .arg("ssh-config")
        .current_dir(folder)
        .stdout(ssh_configuration)
        .status()?
        .success());
    Ok(())
}

fn clean(folder: &path::Path) -> io::Result<process::ExitStatus> {
    process::Command::new(env!("CARGO_BIN_EXE_kerek"))
        .arg("clean")
        .current_dir(folder)
        .status()
}

fn provision(folder: &path::Path) -> io::Result<process::ExitStatus> {
    process::Command::new(env!("CARGO_BIN_EXE_kerek"))
        .arg("provision")
        .current_dir(folder)
        .status()
}

fn run(folder: &path::Path) -> io::Result<process::ExitStatus> {
    process::Command::new(env!("CARGO_BIN_EXE_kerek"))
        .arg("run")
        .current_dir(folder)
        .status()
}
