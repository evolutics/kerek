use anyhow::Context;
use std::fs;
use std::io;
use std::path;

pub fn get(path: path::PathBuf) -> anyhow::Result<Main> {
    let file = fs::File::open(&path).with_context(|| format!("Unable to open file: {path:?}"))?;
    let configuration = serde_json::from_reader(io::BufReader::new(file))?;
    Ok(convert(configuration))
}

pub struct Main {
    pub workspace: WorkspaceConfiguration,
    pub tests: TestsConfiguration,
    pub staging: EnvironmentConfiguration,
    pub production: EnvironmentConfiguration,
}

pub struct WorkspaceConfiguration {
    pub folder: path::PathBuf,
    pub provision: path::PathBuf,
    pub vagrantfile: path::PathBuf,
    pub build: path::PathBuf,
    pub vm_name: String,
    pub vm_snapshot: String,
}

pub struct TestsConfiguration {
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
    pub tests: UserFacingTestsConfiguration,
    pub production: UserFacingProductionConfiguration,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingTestsConfiguration {
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

fn convert(configuration: UserFacingConfiguration) -> Main {
    let workspace_folder = configuration
        .workspace_folder
        .unwrap_or_else(|| path::PathBuf::from(".kerek"));
    let staging = staging_configuration(&workspace_folder);
    let workspace = workspace_configuration(workspace_folder);

    Main {
        workspace,
        tests: TestsConfiguration {
            base: configuration
                .tests
                .base
                .unwrap_or_else(|| ["scripts", "base_test.sh"].iter().collect()),
            acceptance: configuration
                .tests
                .acceptance
                .unwrap_or_else(|| ["scripts", "acceptance_test.sh"].iter().collect()),
            smoke: configuration
                .tests
                .smoke
                .unwrap_or_else(|| ["scripts", "smoke_test.sh"].iter().collect()),
        },
        staging,
        production: EnvironmentConfiguration {
            ssh_configuration_file: configuration
                .production
                .ssh_configuration
                .unwrap_or_else(|| ["safe", "ssh_configuration"].iter().collect()),
            ssh_host: configuration.production.ssh_host,
            kubeconfig_file: configuration
                .production
                .kubeconfig
                .unwrap_or_else(|| ["safe", "kubeconfig"].iter().collect()),
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
        vm_name: String::from("default"),
        vm_snapshot: String::from("default"),
    }
}
