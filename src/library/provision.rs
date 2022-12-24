use super::command;
use crate::library::configuration;
use std::process;

pub fn go(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    command::status(
        process::Command::new(&configuration.life_cycle.provision[0])
            .args(&configuration.life_cycle.provision[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
}
