use crate::library::configuration;
use crate::library::provision;
use crate::library::set_up_workspace;
use std::path;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(configuration)?;

    set_up_workspace::go(&configuration.work_folder)?;

    provision::go(provision::In {
        scripts: &configuration.provisioning_scripts,
        ssh_configuration_file: &configuration.production.ssh_configuration_file,
        ssh_host: &configuration.production.ssh_host,
        kubeconfig_file: &configuration.production.kubeconfig_file,
        public_ip: &configuration.production.public_ip,
    })
}
