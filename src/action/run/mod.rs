mod iterate;
mod set_up;

use crate::library::clean;
use crate::library::configuration;
use crate::library::loop_until_sigint;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;

    loop_until_sigint::go(loop_until_sigint::In {
        set_up: || set_up::go(&configuration),
        iterate: || iterate::go(&configuration),
        tear_down: || clean::go(&configuration.work_folder).expect("Unable to clean."),
    })
}
