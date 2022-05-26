mod iterate;
mod reset;

use crate::library::configuration;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;

    loop {
        reset::go(&configuration)?;
        iterate::go(&configuration)?;
    }
}
