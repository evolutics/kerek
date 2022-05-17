use crate::library::configuration;
use crate::library::tear_down_cache;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;
    tear_down_cache::go(&configuration.cache)
}
