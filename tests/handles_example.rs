use std::env;
use std::process;

#[test]
fn go() -> anyhow::Result<()> {
    assert!(process::Command::new("tests/handles_example.sh")
        .env("WHEELSTICKS", env!("CARGO_BIN_EXE_wheelsticks"))
        .status()?
        .success());
    Ok(())
}
