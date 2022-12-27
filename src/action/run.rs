use crate::library::configuration;
use crate::library::run;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;
    run::go(&configuration)
}
