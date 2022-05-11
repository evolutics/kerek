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

    pub base_test: Option<String>,
    pub acceptance_test: Option<String>,
    pub smoke_test: Option<String>,

    pub ssh_configuration: Option<String>,
    pub ssh_host: String,
    pub kubeconfig: Option<String>,
    pub public_ip: String,
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
            base_test: configuration
                .base_test
                .unwrap_or_else(|| String::from("scripts/base_test.sh")),
            acceptance_test: configuration
                .acceptance_test
                .unwrap_or_else(|| String::from("scripts/acceptance_test.sh")),
            smoke_test: configuration
                .smoke_test
                .unwrap_or_else(|| String::from("scripts/smoke_test.sh")),
            staging: EnvironmentConfiguration {
                ssh_configuration_file: format!("{work_folder}/ssh_configuration"),
                ssh_host: String::from("default"),
                kubeconfig_file: format!("{work_folder}/kubeconfig"),
                public_ip: String::from("192.168.63.63"),
            },
            production: EnvironmentConfiguration {
                ssh_configuration_file: configuration
                    .ssh_configuration
                    .unwrap_or_else(|| String::from("safe/ssh_configuration")),
                ssh_host: configuration.ssh_host,
                kubeconfig_file: configuration
                    .kubeconfig
                    .unwrap_or_else(|| String::from("safe/kubeconfig")),
                public_ip: configuration.public_ip,
            },
        }
    }
}
