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
            .env("KEREK_CACHE_FOLDER", &configuration.cache.folder)
            .env("KEREK_CACHE_WORKBENCH", &configuration.cache.workbench)
            .env("KEREK_IP_ADDRESS", &environment.ip_address)
            .env(
                "KEREK_SSH_CONFIGURATION",
                &environment.ssh_configuration_file,
            )
            .env("KEREK_SSH_HOST", &environment.ssh_host)
            .envs(&configuration.variables)
            .envs(&environment.variables),
    )
}
