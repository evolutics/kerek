use crate::library::configuration;
use crate::library::run;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;
    run::go(
        &configuration,
        run::Options {
            is_dry_run: true,
            is_vm_snapshot_asserted: false,
        },
    )
}
