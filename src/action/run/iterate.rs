use crate::library::command;
use crate::library::configuration;
use std::process;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    command::status(&mut process::Command::new(&configuration.base_test))
}
