use crate::library::configuration;
use crate::library::provision;
use crate::library::set_up_cache;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;

    set_up_cache::go(&configuration)?;

    provision::go(provision::In {
        script_file: &configuration.cache.provision,
        ssh_configuration_file: &configuration.production.ssh_configuration_file,
        ssh_host: &configuration.production.ssh_host,
        kubeconfig_file: &configuration.production.kubeconfig_file,
        ip_address: &configuration.production.ip_address,
    })
}
