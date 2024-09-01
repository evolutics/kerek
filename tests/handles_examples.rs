use std::env;
use std::process;

#[test_case::test_case("examples/ssh_delivery")]
#[test_case::test_case("examples/zero_downtime_deployment")]
fn go(folder: &str) -> anyhow::Result<()> {
    assert!(process::Command::new("./test.sh")
        .env("KEREK", env!("CARGO_BIN_EXE_kerek"))
        .current_dir(folder)
        .status()?
        .success());
    Ok(())
}
