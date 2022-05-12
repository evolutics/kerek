use crate::library::clean;
use crate::library::configuration;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;
    clean::go(&configuration.work_folder)
}
