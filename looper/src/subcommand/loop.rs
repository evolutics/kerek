use crate::library::configuration;
use crate::library::r#loop;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;
    r#loop::go(&configuration, false)
}
