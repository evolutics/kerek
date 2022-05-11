use super::constants;
use anyhow::Context;
use std::fs;
use std::io;

pub fn get() -> anyhow::Result<Main> {
    let path = constants::CONFIGURATION_FILE;
    let file = fs::File::open(path).with_context(|| format!("Unable to open file: {path}"))?;
    let configuration =
        serde_json::from_reader::<_, UserFacingConfiguration>(io::BufReader::new(file))?;
    Ok(configuration.into())
}

pub struct Main {
    pub provisioning_scripts: Vec<String>,
    pub base_test: String,
    pub acceptance_test: String,
    pub smoke_test: String,
    pub staging: EnvironmentConfiguration,
    pub production: EnvironmentConfiguration,
}

pub struct EnvironmentConfiguration {
    pub ssh_configuration_file: String,
    pub ssh_host: String,
    pub kubeconfig_file: String,
    pub public_ip: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingConfiguration {
    pub provision_extras: Option<String>,

    #[serde(default = "default_base_test")]
    pub base_test: String,
    #[serde(default = "default_acceptance_test")]
    pub acceptance_test: String,
    #[serde(default = "default_smoke_test")]
    pub smoke_test: String,

    #[serde(default = "default_ssh_configuration")]
    pub ssh_configuration: String,
    pub ssh_host: String,
    #[serde(default = "default_kubeconfig")]
    pub kubeconfig: String,
    pub public_ip: String,
}

fn default_base_test() -> String {
    String::from("scripts/base_test.sh")
}

fn default_acceptance_test() -> String {
    String::from("scripts/acceptance_test.sh")
}

fn default_smoke_test() -> String {
    String::from("scripts/smoke_test.sh")
}

fn default_ssh_configuration() -> String {
    String::from("safe/ssh_configuration")
}

fn default_kubeconfig() -> String {
    String::from("safe/kubeconfig")
}

impl From<UserFacingConfiguration> for Main {
    fn from(configuration: UserFacingConfiguration) -> Self {
        let work_folder = constants::WORK_FOLDER;

        Self {
            provisioning_scripts: [
                Some(constants::provision_base_file()),
                configuration.provision_extras,
            ]
            .into_iter()
            .flatten()
            .collect(),
            base_test: configuration.base_test,
            acceptance_test: configuration.acceptance_test,
            smoke_test: configuration.smoke_test,
            staging: EnvironmentConfiguration {
                ssh_configuration_file: format!("{work_folder}/ssh_configuration"),
                ssh_host: String::from("default"),
                kubeconfig_file: format!("{work_folder}/kubeconfig"),
                public_ip: String::from("192.168.63.63"),
            },
            production: EnvironmentConfiguration {
                ssh_configuration_file: configuration.ssh_configuration,
                ssh_host: configuration.ssh_host,
                kubeconfig_file: configuration.kubeconfig,
                public_ip: configuration.public_ip,
            },
        }
    }
}
