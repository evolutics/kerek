use anyhow::Context;
use std::env;
use std::iter;
use std::path;
use std::process;

#[test_case::test_case("examples/ssh_delivery")]
#[test_case::test_case("examples/zero_downtime_deployment")]
fn go(folder: &str) -> anyhow::Result<()> {
    let executable_folder = path::Path::new(env!("CARGO_BIN_EXE_kerek"))
        .parent()
        .context("No parent")?;
    let original_path = env::var_os("PATH").unwrap_or_default();
    let path_with_executable_under_test = env::join_paths(
        iter::once(executable_folder.into()).chain(env::split_paths(&original_path)),
    )?;

    assert!(process::Command::new("./test.sh")
        .env("PATH", path_with_executable_under_test)
        .current_dir(folder)
        .status()?
        .success());
    Ok(())
}
