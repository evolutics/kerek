use super::command;
use super::configuration;
use anyhow::Context;
use std::process;

pub fn go(
    configuration: &configuration::Main,
    environment: &configuration::Environment,
) -> anyhow::Result<()> {
    let environment_id = &environment.id;
    crate::log!("Provisioning {environment_id} environment.");
    command::status(
        process::Command::new(&configuration.life_cycle.provision[0])
            .args(&configuration.life_cycle.provision[1..])
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
    .with_context(|| format!("Unable to provision {environment_id} environment."))
}
