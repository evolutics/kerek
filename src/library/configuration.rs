use super::constants;
use anyhow::Context;
use std::fs;
use std::io;
use std::path;

pub fn get(path: path::PathBuf) -> anyhow::Result<Main> {
    let file = fs::File::open(&path).with_context(|| format!("Unable to open file: {path:?}"))?;
    let configuration = serde_json::from_reader(io::BufReader::new(file))?;
    let root = path
        .parent()
        .ok_or_else(|| anyhow::anyhow!("No parent for path: {path:?}"))?;
    Ok(convert(configuration, root))
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

fn convert(configuration: UserFacingConfiguration, root: &path::Path) -> Main {
    Main {
        provisioning_scripts: [
            Some(constants::provision_base_file()),
            configuration.provision_extras,
        ]
        .into_iter()
        .flatten()
        .map(|path| root.join(path))
        .collect(),
        base_test: configuration
            .base_test
            .unwrap_or_else(|| root.join("scripts/base_test.sh")),
        acceptance_test: configuration
            .acceptance_test
            .unwrap_or_else(|| root.join("scripts/acceptance_test.sh")),
        smoke_test: configuration
            .smoke_test
            .unwrap_or_else(|| root.join("scripts/smoke_test.sh")),
        staging: staging(root),
        production: EnvironmentConfiguration {
            ssh_configuration_file: configuration
                .ssh_configuration
                .unwrap_or_else(|| root.join("safe/ssh_configuration")),
            ssh_host: configuration.ssh_host,
            kubeconfig_file: configuration
                .kubeconfig
                .unwrap_or_else(|| root.join("safe/kubeconfig")),
            public_ip: configuration.public_ip,
        },
    }
}

fn staging(root: &path::Path) -> EnvironmentConfiguration {
    let work_folder = constants::WORK_FOLDER;
    EnvironmentConfiguration {
        ssh_configuration_file: root.join(format!("{work_folder}/ssh_configuration")),
        ssh_host: String::from("default"),
        kubeconfig_file: root.join(format!("{work_folder}/kubeconfig")),
        public_ip: String::from("192.168.63.63"),
    }
}
