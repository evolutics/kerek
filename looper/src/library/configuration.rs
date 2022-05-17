use anyhow::Context;
use std::ffi;
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
    pub life_cycle: LifeCycleConfiguration,
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
    pub base: Vec<ffi::OsString>,
    pub smoke: Vec<ffi::OsString>,
    pub acceptance: Vec<ffi::OsString>,
}

pub struct EnvironmentConfiguration {
    pub ssh_configuration_file: path::PathBuf,
    pub ssh_host: String,
    pub kubeconfig_file: path::PathBuf,
    pub public_ip: String,
}

pub struct LifeCycleConfiguration {
    pub move_to_next_version: Vec<ffi::OsString>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingConfiguration {
    pub workspace_folder: Option<path::PathBuf>,
    #[serde(default)]
    pub tests: UserFacingTestsConfiguration,
    pub production: UserFacingProductionConfiguration,
    #[serde(default)]
    pub life_cycle: UserFacingLifeCycleConfiguration,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingTestsConfiguration {
    #[serde(default)]
    pub base: Vec<String>,
    #[serde(default)]
    pub smoke: Vec<String>,
    #[serde(default)]
    pub acceptance: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingProductionConfiguration {
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: String,
    pub kubeconfig: Option<path::PathBuf>,
    pub public_ip: String,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingLifeCycleConfiguration {
    #[serde(default)]
    pub move_to_next_version: Vec<String>,
}

fn convert(configuration: UserFacingConfiguration) -> Main {
    let workspace_folder = configuration
        .workspace_folder
        .unwrap_or_else(|| path::PathBuf::from(".kerek"));

    let workspace = workspace_configuration(workspace_folder);
    let tests = tests_configuration(configuration.tests);
    let staging = staging_configuration(&workspace.folder);
    let production = production_configuration(configuration.production);
    let life_cycle = life_cycle_configuration(configuration.life_cycle);

    Main {
        workspace,
        tests,
        staging,
        production,
        life_cycle,
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

fn tests_configuration(tests: UserFacingTestsConfiguration) -> TestsConfiguration {
    TestsConfiguration {
        base: convert_nonempty_or_else(tests.base, || {
            vec![ffi::OsString::from("scripts/base_test.sh")]
        }),
        smoke: convert_nonempty_or_else(tests.smoke, || {
            vec![ffi::OsString::from("scripts/smoke_test.sh")]
        }),
        acceptance: convert_nonempty_or_else(tests.acceptance, || {
            vec![ffi::OsString::from("scripts/acceptance_test.sh")]
        }),
    }
}

fn convert_nonempty_or_else<F: Fn() -> Vec<U>, T, U: From<T>>(
    sequence: Vec<T>,
    if_empty: F,
) -> Vec<U> {
    if sequence.is_empty() {
        if_empty()
    } else {
        sequence.into_iter().map(|element| element.into()).collect()
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

fn production_configuration(
    production: UserFacingProductionConfiguration,
) -> EnvironmentConfiguration {
    EnvironmentConfiguration {
        ssh_configuration_file: production
            .ssh_configuration
            .unwrap_or_else(|| ["safe", "ssh_configuration"].iter().collect()),
        ssh_host: production.ssh_host,
        kubeconfig_file: production
            .kubeconfig
            .unwrap_or_else(|| ["safe", "kubeconfig"].iter().collect()),
        public_ip: production.public_ip,
    }
}

fn life_cycle_configuration(
    life_cycle: UserFacingLifeCycleConfiguration,
) -> LifeCycleConfiguration {
    LifeCycleConfiguration {
        move_to_next_version: convert_nonempty_or_else(life_cycle.move_to_next_version, || {
            [
                "bash",
                "-c",
                "--",
                include_str!("assets/move_to_next_version.sh"),
            ]
            .iter()
            .map(ffi::OsString::from)
            .collect()
        }),
    }
}
