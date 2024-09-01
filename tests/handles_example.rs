use std::env;
use std::process;

#[test]
fn go() -> anyhow::Result<()> {
    assert!(process::Command::new("tests/handles_example.sh")
        .env("KEREK", env!("CARGO_BIN_EXE_kerek"))
        .status()?
        .success());
    Ok(())
}
