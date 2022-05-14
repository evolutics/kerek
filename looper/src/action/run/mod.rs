mod iterate;
mod set_up;

use crate::library::configuration;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;
    set_up::go(&configuration)?;

    loop {
        iterate::go(&configuration)?;
    }
}
