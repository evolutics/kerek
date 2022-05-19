use anyhow::Context;
use std::ffi;
use std::fs;
use std::io;
use std::path;

pub fn get(path: path::PathBuf) -> anyhow::Result<Main> {
    let file = fs::File::open(&path).with_context(|| format!("Unable to open file: {path:?}"))?;
    let main = serde_json::from_reader(io::BufReader::new(file))?;
    Ok(convert(main))
}

pub struct Main {
    pub cache: Cache,
    pub tests: Tests,
    pub staging: Environment,
    pub production: Environment,
    pub life_cycle: LifeCycle,
}

pub struct Cache {
    pub folder: path::PathBuf,
    pub provision: path::PathBuf,
    pub vagrantfile: path::PathBuf,
    pub vm_name: String,
    pub vm_snapshot: String,
}

pub struct Tests {
    pub base: Vec<ffi::OsString>,
    pub smoke: Vec<ffi::OsString>,
    pub acceptance: Vec<ffi::OsString>,
}

pub struct Environment {
    pub ssh_configuration_file: path::PathBuf,
    pub ssh_host: String,
    pub kubeconfig_file: path::PathBuf,
    pub public_ip: String,
}

pub struct LifeCycle {
    pub build: Vec<ffi::OsString>,
    pub deploy: Vec<ffi::OsString>,
    pub move_to_next_version: Vec<ffi::OsString>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingMain {
    pub cache_folder: Option<path::PathBuf>,
    #[serde(default)]
    pub tests: UserFacingTests,
    pub production: UserFacingProduction,
    #[serde(default)]
    pub life_cycle: UserFacingLifeCycle,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingTests {
    #[serde(default)]
    pub base: Vec<String>,
    #[serde(default)]
    pub smoke: Vec<String>,
    #[serde(default)]
    pub acceptance: Vec<String>,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingProduction {
    pub ssh_configuration: Option<path::PathBuf>,
    pub ssh_host: String,
    pub kubeconfig: Option<path::PathBuf>,
    pub public_ip: String,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingLifeCycle {
    #[serde(default)]
    pub build: Vec<String>,
    #[serde(default)]
    pub deploy: Vec<String>,
    #[serde(default)]
    pub move_to_next_version: Vec<String>,
}

fn convert(main: UserFacingMain) -> Main {
    let cache = get_cache(
        main.cache_folder
            .unwrap_or_else(|| path::PathBuf::from(".kerek")),
    );
    let tests = get_tests(main.tests);
    let staging = get_staging(&cache.folder);
    let production = get_production(main.production);
    let life_cycle = get_life_cycle(main.life_cycle);

    Main {
        cache,
        tests,
        staging,
        production,
        life_cycle,
    }
}

fn get_cache(folder: path::PathBuf) -> Cache {
    let provision = folder.join("provision.sh");
    let vagrantfile = folder.join("Vagrantfile");

    Cache {
        folder,
        provision,
        vagrantfile,
        vm_name: String::from("default"),
        vm_snapshot: String::from("default"),
    }
}

fn get_tests(tests: UserFacingTests) -> Tests {
    Tests {
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

fn get_staging(cache_folder: &path::Path) -> Environment {
    Environment {
        ssh_configuration_file: cache_folder.join("ssh_configuration"),
        ssh_host: String::from("default"),
        kubeconfig_file: cache_folder.join("kubeconfig"),
        public_ip: String::from("192.168.63.63"),
    }
}

fn get_production(production: UserFacingProduction) -> Environment {
    Environment {
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

fn get_life_cycle(life_cycle: UserFacingLifeCycle) -> LifeCycle {
    LifeCycle {
        build: convert_nonempty_or_else(life_cycle.build, || {
            ["bash", "-c", "--", include_str!("assets/build.sh")]
                .iter()
                .map(ffi::OsString::from)
                .collect()
        }),
        deploy: convert_nonempty_or_else(life_cycle.deploy, || {
            ["bash", "-c", "--", include_str!("assets/deploy.sh")]
                .iter()
                .map(ffi::OsString::from)
                .collect()
        }),
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
