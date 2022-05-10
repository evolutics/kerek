use super::constants;
use anyhow::Context;
use std::fs;
use std::io;

pub fn get() -> anyhow::Result<Data> {
    let path = constants::CONFIGURATION_FILE;
    let file = fs::File::open(path).with_context(|| format!("Unable to open file: {path}"))?;
    let configuration =
        serde_json::from_reader::<_, UserFacingConfiguration>(io::BufReader::new(file))?;
    Ok(configuration.into())
}

pub struct Data {
    pub provision_extras: String,
    pub test_base: String,
    pub staging: EnvironmentConfiguration,
    pub production: EnvironmentConfiguration,
}

pub struct EnvironmentConfiguration {
    pub ssh_configuration_file: String,
    pub ssh_host: String,
    pub kubeconfig_file: String,
    pub public_ip: String,
    pub test_file: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingConfiguration {
    #[serde(default = "default_provision_extras")]
    pub provision_extras: String,

    #[serde(default = "default_test_base")]
    pub test_base: String,
    #[serde(default = "default_test_staging")]
    pub test_staging: String,
    #[serde(default = "default_test_production")]
    pub test_production: String,

    #[serde(default = "default_production_ssh_configuration")]
    pub production_ssh_configuration: String,
    pub production_ssh_host: String,
    #[serde(default = "default_production_kubeconfig")]
    pub production_kubeconfig: String,
    pub production_public_ip: String,
}

fn default_provision_extras() -> String {
    String::from("scripts/provision_extras.sh")
}

fn default_test_base() -> String {
    String::from("scripts/test_base.sh")
}

fn default_test_staging() -> String {
    String::from("scripts/test_staging.sh")
}

fn default_test_production() -> String {
    String::from("scripts/test_production.sh")
}

fn default_production_ssh_configuration() -> String {
    String::from("safe/production_ssh_configuration")
}

fn default_production_kubeconfig() -> String {
    String::from("safe/production_kubeconfig")
}

impl From<UserFacingConfiguration> for Data {
    fn from(configuration: UserFacingConfiguration) -> Self {
        let work_folder = constants::WORK_FOLDER;

        Data {
            provision_extras: configuration.provision_extras,
            test_base: configuration.test_base,
            staging: EnvironmentConfiguration {
                ssh_configuration_file: format!("{work_folder}/ssh_configuration"),
                ssh_host: String::from("default"),
                kubeconfig_file: format!("{work_folder}/kubeconfig"),
                public_ip: String::from("192.168.63.63"),
                test_file: configuration.test_staging,
            },
            production: EnvironmentConfiguration {
                ssh_configuration_file: configuration.production_ssh_configuration,
                ssh_host: configuration.production_ssh_host,
                kubeconfig_file: configuration.production_kubeconfig,
                public_ip: configuration.production_public_ip,
                test_file: configuration.test_production,
            },
        }
    }
}
