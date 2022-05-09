use super::configuration;
use super::constants;
use super::run_bash_script_over_ssh;

pub fn go(configuration: &configuration::Data, in_: In) -> anyhow::Result<()> {
    provision_base(&in_)?;
    provision_extras(configuration, &in_)
}

pub struct In<'a> {
    pub ssh_configuration_file: &'a str,
    pub ssh_hostname: &'a str,
}

fn provision_base(in_: &In) -> anyhow::Result<()> {
    run_bash_script_over_ssh::go(run_bash_script_over_ssh::In {
        configuration_file: in_.ssh_configuration_file,
        hostname: in_.ssh_hostname,
        script_file: &constants::provision_base_file(),
    })
}

fn provision_extras(configuration: &configuration::Data, in_: &In) -> anyhow::Result<()> {
    run_bash_script_over_ssh::go(run_bash_script_over_ssh::In {
        configuration_file: in_.ssh_configuration_file,
        hostname: in_.ssh_hostname,
        script_file: &configuration.provision_extras,
    })
}
