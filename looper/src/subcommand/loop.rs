use crate::library::command;
use crate::library::configuration;
use crate::library::run;
use anyhow::Context;
use std::path;
use std::process;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;

    for iteration in 0.. {
        crate::log!("Executing iteration number {iteration}.");

        run::go(
            &configuration,
            run::Options {
                is_dry_run: false,
                is_vm_snapshot_asserted: iteration != 0,
            },
        )?;

        move_to_next_version(&configuration)?;
    }

    Ok(())
}

fn move_to_next_version(configuration: &configuration::Main) -> anyhow::Result<()> {
    crate::log!("Moving to next version.");
    command::status(
        process::Command::new(&configuration.life_cycle.move_to_next_version[0])
            .args(&configuration.life_cycle.move_to_next_version[1..])
            .envs(&configuration.variables),
    )
    .context("Unable to move to next version.")
}
