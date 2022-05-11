use super::constants;
use anyhow::Context;
use std::fs;
use std::io;
use std::path;

pub fn get() -> anyhow::Result<Main> {
    let path = constants::CONFIGURATION_FILE;
    let file = fs::File::open(path).with_context(|| format!("Unable to open file: {path}"))?;
    let configuration =
        serde_json::from_reader::<_, UserFacingConfiguration>(io::BufReader::new(file))?;
    Ok(configuration.into())
}

pub struct Main {
    pub provisioning_scripts: Vec<path::PathBuf>,
    pub base_test: path::PathBuf,
    pub acceptance_test: path::PathBuf,
    pub smoke_test: path::PathBuf,
    pub staging: EnvironmentConfiguration,
    pub production: EnvironmentConfiguration,
}

pub struct EnvironmentConfiguration {
    pub ssh_configuration_file: path::PathBuf,
    pub ssh_host: String,
    pub kubeconfig_file: path::PathBuf,
    pub public_ip: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingConfiguration {
    pub provision_extras: Option<path::PathBuf>,

    pub base_test: Option<path::PathBuf>,
    pub acceptance_test: Option<path::PathBuf>,
    pub smoke_test: Option<path::PathBuf>,

    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: String,
    pub kubeconfig: Option<path::PathBuf>,
    pub public_ip: String,
}

impl From<UserFacingConfiguration> for Main {
    fn from(configuration: UserFacingConfiguration) -> Self {
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
                .unwrap_or_else(|| ["scripts", "base_test.sh"].into_iter().collect()),
            acceptance_test: configuration
                .acceptance_test
                .unwrap_or_else(|| ["scripts", "acceptance_test.sh"].into_iter().collect()),
            smoke_test: configuration
                .smoke_test
                .unwrap_or_else(|| ["scripts", "smoke_test.sh"].into_iter().collect()),
            staging: EnvironmentConfiguration {
                ssh_configuration_file: [constants::WORK_FOLDER, "ssh_configuration"]
                    .into_iter()
                    .collect(),
                ssh_host: String::from("default"),
                kubeconfig_file: [constants::WORK_FOLDER, "kubeconfig"].into_iter().collect(),
                public_ip: String::from("192.168.63.63"),
            },
            production: EnvironmentConfiguration {
                ssh_configuration_file: configuration
                    .ssh_configuration
                    .unwrap_or_else(|| ["safe", "ssh_configuration"].into_iter().collect()),
                ssh_host: configuration.ssh_host,
                kubeconfig_file: configuration
                    .kubeconfig
                    .unwrap_or_else(|| ["safe", "kubeconfig"].into_iter().collect()),
                public_ip: configuration.public_ip,
            },
        }
    }
}
