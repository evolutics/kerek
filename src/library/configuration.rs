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
    pub workspace: WorkspaceConfiguration,
    pub test: TestConfiguration,
    pub staging: EnvironmentConfiguration,
    pub production: EnvironmentConfiguration,
}

pub struct WorkspaceConfiguration {
    pub folder: path::PathBuf,
    pub provision: path::PathBuf,
    pub vagrantfile: path::PathBuf,
    pub build: path::PathBuf,
}

pub struct TestConfiguration {
    pub base: path::PathBuf,
    pub acceptance: path::PathBuf,
    pub smoke: path::PathBuf,
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
    pub workspace_folder: Option<path::PathBuf>,
    #[serde(default)]
    pub test: UserFacingTestConfiguration,
    pub production: UserFacingProductionConfiguration,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingTestConfiguration {
    pub base: Option<path::PathBuf>,
    pub acceptance: Option<path::PathBuf>,
    pub smoke: Option<path::PathBuf>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingProductionConfiguration {
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: String,
    pub kubeconfig: Option<path::PathBuf>,
    pub public_ip: String,
}

fn convert(configuration: UserFacingConfiguration, root: &path::Path) -> Main {
    let workspace_folder = root.join(
        configuration
            .workspace_folder
            .unwrap_or_else(|| path::PathBuf::from(".kerek")),
    );
    let staging = staging_configuration(&workspace_folder);
    let workspace = workspace_configuration(workspace_folder);

    Main {
        workspace,
        test: TestConfiguration {
            base: root.join(
                configuration
                    .test
                    .base
                    .unwrap_or_else(|| ["scripts", "base_test.sh"].iter().collect()),
            ),
            acceptance: root.join(
                configuration
                    .test
                    .acceptance
                    .unwrap_or_else(|| ["scripts", "acceptance_test.sh"].iter().collect()),
            ),
            smoke: root.join(
                configuration
                    .test
                    .smoke
                    .unwrap_or_else(|| ["scripts", "smoke_test.sh"].iter().collect()),
            ),
        },
        staging,
        production: EnvironmentConfiguration {
            ssh_configuration_file: root.join(
                configuration
                    .production
                    .ssh_configuration
                    .unwrap_or_else(|| ["safe", "ssh_configuration"].iter().collect()),
            ),
            ssh_host: configuration.production.ssh_host,
            kubeconfig_file: root.join(
                configuration
                    .production
                    .kubeconfig
                    .unwrap_or_else(|| ["safe", "kubeconfig"].iter().collect()),
            ),
            public_ip: configuration.production.public_ip,
        },
    }
}

fn staging_configuration(workspace_folder: &path::Path) -> EnvironmentConfiguration {
    EnvironmentConfiguration {
        ssh_configuration_file: workspace_folder.join("ssh_configuration"),
        ssh_host: String::from("default"),
        kubeconfig_file: workspace_folder.join("kubeconfig"),
        public_ip: String::from("192.168.63.63"),
    }
}

fn workspace_configuration(folder: path::PathBuf) -> WorkspaceConfiguration {
    let provision = folder.join("provision.sh");
    let vagrantfile = folder.join("Vagrantfile");
    let build = folder.join("build.json");

    WorkspaceConfiguration {
        folder,
        provision,
        vagrantfile,
        build,
    }
}
