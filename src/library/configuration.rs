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
    pub test_staging: String,
    pub test_production: String,
    pub production_kubeconfig: String,
}

#[derive(Debug, serde::Deserialize)]
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

    #[allow(dead_code)]
    #[serde(default = "default_production_ssh_configuration")]
    pub production_ssh_configuration: String,
    #[allow(dead_code)]
    pub production_ssh_host: String,
    #[serde(default = "default_production_kubeconfig")]
    pub production_kubeconfig: String,
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
        Data {
            provision_extras: configuration.provision_extras,
            test_base: configuration.test_base,
            test_staging: configuration.test_staging,
            test_production: configuration.test_production,
            production_kubeconfig: configuration.production_kubeconfig,
        }
    }
}
