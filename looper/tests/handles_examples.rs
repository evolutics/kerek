use std::fs;
use std::io;
use std::path;
use std::process;

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

fn execute_subcommand(subcommand: &str, folder: &path::Path) -> io::Result<process::ExitStatus> {
    process::Command::new(env!("CARGO_BIN_EXE_kerek"))
        .arg(subcommand)
        .current_dir(folder)
        .status()
}
