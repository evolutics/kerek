use super::super::configuration;
use super::super::provision;
use super::super::set_up_cache;
use super::super::tear_down_cache;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    tear_down_cache::go(configuration)?;
    set_up_cache::go(configuration, true)?;
    provision::go(configuration, &configuration.staging)
}
