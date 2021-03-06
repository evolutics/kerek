use anyhow::Context;
use std::collections::hash_map;
use std::ffi;
use std::fs;
use std::hash::Hash;
use std::hash::Hasher;
use std::io;
use std::path;
use std::time;

pub fn get(path: path::PathBuf) -> anyhow::Result<Main> {
    let file = fs::File::open(&path).with_context(|| format!("Unable to open file: {path:?}"))?;
    let main = serde_json::from_reader(io::BufReader::new(file))?;
    Ok(convert(main))
}

pub struct Main {
    pub cache: Cache,
    pub life_cycle: LifeCycle,
    pub tests: Tests,
    pub staging: Environment,
    pub production: Environment,
}

pub struct Cache {
    pub folder: path::PathBuf,
    pub provision: path::PathBuf,
    pub vagrantfile: path::PathBuf,
}

pub struct LifeCycle {
    pub build: Vec<ffi::OsString>,
    pub deploy: Vec<ffi::OsString>,
    pub move_to_next_version: Vec<ffi::OsString>,
}

pub struct Tests {
    pub base: Vec<ffi::OsString>,
    pub smoke: Vec<ffi::OsString>,
    pub acceptance: Vec<ffi::OsString>,
}

pub struct Environment {
    pub display_name: String,
    pub ssh_configuration_file: path::PathBuf,
    pub ssh_host: String,
    pub kubeconfig_file: path::PathBuf,
    pub ip_address: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingMain {
    pub cache_folder: Option<path::PathBuf>,
    #[serde(default)]
    pub life_cycle: UserFacingLifeCycle,
    #[serde(default)]
    pub tests: UserFacingTests,
    pub production: UserFacingProduction,
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
    pub ip_address: String,
}

fn convert(main: UserFacingMain) -> Main {
    let cache = get_cache(
        main.cache_folder
            .unwrap_or_else(|| path::PathBuf::from(".kerek")),
    );
    let life_cycle = get_life_cycle(main.life_cycle);
    let tests = get_tests(main.tests);
    let staging = get_staging(&cache);
    let production = get_production(main.production);

    Main {
        cache,
        life_cycle,
        tests,
        staging,
        production,
    }
}

fn get_cache(folder: path::PathBuf) -> Cache {
    let provision = folder.join("provision.sh");
    let vagrantfile = folder.join("Vagrantfile");

    Cache {
        folder,
        provision,
        vagrantfile,
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
            ["python3", "-c", include_str!("assets/deploy.py")]
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

fn get_staging(cache: &Cache) -> Environment {
    Environment {
        display_name: String::from("staging"),
        ssh_configuration_file: cache.folder.join("ssh_configuration"),
        ssh_host: String::from("default"),
        kubeconfig_file: cache.folder.join("kubeconfig"),
        ip_address: get_staging_ip_address(cache),
    }
}

fn get_staging_ip_address(cache: &Cache) -> String {
    let existing_ip_address = match fs::read_to_string(&cache.vagrantfile) {
        Err(_) => None,
        Ok(full_contents) => full_contents
            .split_once(" ip: \"")
            .and_then(|(_, contents_from_ip_address)| contents_from_ip_address.split_once('"'))
            .map(|(ip_address, _)| String::from(ip_address)),
    };

    existing_ip_address.unwrap_or_else(|| {
        let random = draw_random();
        format!("192.168.60.{random}")
    })
}

fn draw_random() -> u8 {
    let mut hasher = hash_map::DefaultHasher::new();
    time::Instant::now().hash(&mut hasher);
    hasher.finish().to_le_bytes()[0]
}

fn get_production(production: UserFacingProduction) -> Environment {
    Environment {
        display_name: String::from("production"),
        ssh_configuration_file: production
            .ssh_configuration
            .unwrap_or_else(|| ["safe", "ssh_configuration"].iter().collect()),
        ssh_host: production.ssh_host,
        kubeconfig_file: production
            .kubeconfig
            .unwrap_or_else(|| ["safe", "kubeconfig"].iter().collect()),
        ip_address: production.ip_address,
    }
}
