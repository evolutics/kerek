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
    pub work_folder: path::PathBuf,
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
    pub work_folder: Option<path::PathBuf>,

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
    let work_folder = root.join(
        configuration
            .work_folder
            .unwrap_or_else(|| path::PathBuf::from(".kerek")),
    );
    let provisioning_scripts = [
        Some(work_folder.join(constants::PROVISION_BASE_FILENAME)),
        configuration.provision_extras.map(|path| root.join(path)),
    ]
    .into_iter()
    .flatten()
    .collect();
    let staging = staging_configuration(&work_folder);

    Main {
        work_folder,
        provisioning_scripts,
        base_test: root.join(
            configuration
                .base_test
                .unwrap_or_else(|| ["scripts", "base_test.sh"].iter().collect()),
        ),
        acceptance_test: root.join(
            configuration
                .acceptance_test
                .unwrap_or_else(|| ["scripts", "acceptance_test.sh"].iter().collect()),
        ),
        smoke_test: root.join(
            configuration
                .smoke_test
                .unwrap_or_else(|| ["scripts", "smoke_test.sh"].iter().collect()),
        ),
        staging,
        production: EnvironmentConfiguration {
            ssh_configuration_file: root.join(
                configuration
                    .ssh_configuration
                    .unwrap_or_else(|| ["safe", "ssh_configuration"].iter().collect()),
            ),
            ssh_host: configuration.ssh_host,
            kubeconfig_file: root.join(
                configuration
                    .kubeconfig
                    .unwrap_or_else(|| ["safe", "kubeconfig"].iter().collect()),
            ),
            public_ip: configuration.public_ip,
        },
    }
}

fn staging_configuration(work_folder: &path::Path) -> EnvironmentConfiguration {
    EnvironmentConfiguration {
        ssh_configuration_file: work_folder.join("ssh_configuration"),
        ssh_host: String::from("default"),
        kubeconfig_file: work_folder.join("kubeconfig"),
        public_ip: String::from("192.168.63.63"),
    }
}
