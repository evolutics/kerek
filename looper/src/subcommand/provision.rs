use crate::library::configuration;
use crate::library::provision;
use crate::library::set_up_cache;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;
    set_up_cache::go(&configuration)?;
    provision::go(&configuration, &configuration.production)
}
