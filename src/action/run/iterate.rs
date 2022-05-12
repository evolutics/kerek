use crate::library::configuration;
use crate::library::run_command;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    run_command::go(&mut process::Command::new(&configuration.base_test))
}
