use std::fs;
use std::io;
use std::path;
use std::process;

const FOLDER: &str = "example";

#[test]
fn go() -> anyhow::Result<()> {
    reset_fake_production()?;
    let log_file = path::PathBuf::from(format!("{FOLDER}/log.txt"));
    fs::write(&log_file, "")?;

    assert!(clean()?.success());
    assert!(provision()?.success());
    assert!(!run()?.success());

    assert_eq!(
        fs::read_to_string(log_file)?,
        "Base tests
Smoke tests
Acceptance tests
Smoke tests
Move to next version
Base tests
",
    );

    Ok(())
}

fn reset_fake_production() -> anyhow::Result<()> {
    assert!(process::Command::new("vagrant")
        .arg("destroy")
        .arg("--force")
        .current_dir(FOLDER)
        .status()?
        .success());
    assert!(process::Command::new("vagrant")
        .arg("up")
        .current_dir(FOLDER)
        .status()?
        .success());
    let ssh_configuration =
        fs::File::create(path::Path::new(&format!("{FOLDER}/safe/ssh_configuration")))?;
    assert!(process::Command::new("vagrant")
        .arg("ssh-config")
        .current_dir(FOLDER)
        .stdout(ssh_configuration)
        .status()?
        .success());
    Ok(())
}

fn clean() -> io::Result<process::ExitStatus> {
    process::Command::new(env!("CARGO_BIN_EXE_kerek"))
        .arg("clean")
        .current_dir(FOLDER)
        .status()
}

fn provision() -> io::Result<process::ExitStatus> {
    process::Command::new(env!("CARGO_BIN_EXE_kerek"))
        .arg("provision")
        .current_dir(FOLDER)
        .status()
}

fn run() -> io::Result<process::ExitStatus> {
    process::Command::new(env!("CARGO_BIN_EXE_kerek"))
        .arg("run")
        .current_dir(FOLDER)
        .status()
}
