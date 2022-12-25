use anyhow::Context;
use std::collections;
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
    pub life_cycle: LifeCycle,
    pub tests: Tests,
    pub variables: collections::HashMap<ffi::OsString, ffi::OsString>,
    pub staging: Environment,
    pub production: Environment,
}

pub struct Cache {
    pub folder: path::PathBuf,
    pub build: path::PathBuf,
    pub deploy: path::PathBuf,
    pub deploy_on_remote: path::PathBuf,
    pub move_to_next_version: path::PathBuf,
    pub provision: path::PathBuf,
    pub provision_on_remote: path::PathBuf,
    pub vagrantfile: path::PathBuf,
    pub workbench: path::PathBuf,
}

pub struct LifeCycle {
    pub provision: Vec<ffi::OsString>,
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
    pub id: String,
    pub variables: collections::HashMap<ffi::OsString, ffi::OsString>,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingMain {
    pub cache_folder: Option<path::PathBuf>,
    #[serde(default)]
    pub life_cycle: UserFacingLifeCycle,
    #[serde(default)]
    pub tests: UserFacingTests,
    #[serde(default)]
    pub environment_variables: UserFacingEnvironmentVariables,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingLifeCycle {
    #[serde(default)]
    pub provision: Vec<String>,
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

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingEnvironmentVariables {
    #[serde(default)]
    pub staging: collections::HashMap<String, String>,
    #[serde(default)]
    pub production: collections::HashMap<String, String>,
}

fn convert(main: UserFacingMain) -> Main {
    let cache = get_cache(
        main.cache_folder
            .unwrap_or_else(|| path::PathBuf::from(".kerek")),
    );
    let life_cycle = get_life_cycle(&cache, main.life_cycle);
    let tests = get_tests(main.tests);
    let variables = get_variables(&cache);
    let staging = get_staging(&cache, main.environment_variables.staging);
    let production = get_production(main.environment_variables.production);

    Main {
        cache,
        life_cycle,
        tests,
        variables,
        staging,
        production,
    }
}

fn get_cache(folder: path::PathBuf) -> Cache {
    let build = folder.join("build.py");
    let deploy = folder.join("deploy.py");
    let deploy_on_remote = folder.join("deploy_on_remote.py");
    let move_to_next_version = folder.join("move_to_next_version.sh");
    let provision = folder.join("provision.py");
    let provision_on_remote = folder.join("provision_on_remote.sh");
    let vagrantfile = folder.join("Vagrantfile");
    let workbench = folder.join("workbench");

    Cache {
        folder,
        build,
        deploy,
        deploy_on_remote,
        move_to_next_version,
        provision,
        provision_on_remote,
        vagrantfile,
        workbench,
    }
}

fn get_life_cycle(cache: &Cache, life_cycle: UserFacingLifeCycle) -> LifeCycle {
    LifeCycle {
        provision: convert_nonempty_or_else(life_cycle.provision, || {
            vec![
                ffi::OsString::from("python3"),
                ffi::OsString::from("--"),
                ffi::OsString::from(&cache.provision),
            ]
        }),
        build: convert_nonempty_or_else(life_cycle.build, || {
            vec![
                ffi::OsString::from("python3"),
                ffi::OsString::from("--"),
                ffi::OsString::from(&cache.build),
            ]
        }),
        deploy: convert_nonempty_or_else(life_cycle.deploy, || {
            vec![
                ffi::OsString::from("python3"),
                ffi::OsString::from("--"),
                ffi::OsString::from(&cache.deploy),
            ]
        }),
        move_to_next_version: convert_nonempty_or_else(life_cycle.move_to_next_version, || {
            vec![
                ffi::OsString::from("bash"),
                ffi::OsString::from("--"),
                ffi::OsString::from(&cache.move_to_next_version),
            ]
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

fn get_variables(cache: &Cache) -> collections::HashMap<ffi::OsString, ffi::OsString> {
    collections::HashMap::from([
        (
            ffi::OsString::from("KEREK_CACHE_FOLDER"),
            cache.folder.clone().into_os_string(),
        ),
        (
            ffi::OsString::from("KEREK_CACHE_WORKBENCH"),
            cache.workbench.clone().into_os_string(),
        ),
    ])
}

fn get_staging(
    cache: &Cache,
    custom_variables: collections::HashMap<String, String>,
) -> Environment {
    with_custom_variables(
        Environment {
            id: String::from("staging"),
            variables: collections::HashMap::from([
                (
                    ffi::OsString::from("KEREK_IP_ADDRESS"),
                    ffi::OsString::from("192.168.60.158"),
                ),
                (
                    ffi::OsString::from("KEREK_SSH_CONFIGURATION"),
                    cache.folder.join("ssh_configuration").into(),
                ),
                (
                    ffi::OsString::from("KEREK_SSH_HOST"),
                    ffi::OsString::from("default"),
                ),
            ]),
        },
        custom_variables,
    )
}

fn with_custom_variables(
    environment: Environment,
    custom_variables: collections::HashMap<String, String>,
) -> Environment {
    let mut variables = environment.variables;
    variables.extend(
        custom_variables
            .into_iter()
            .map(|(key, value)| (key.into(), value.into())),
    );

    Environment {
        variables,
        ..environment
    }
}

fn get_production(custom_variables: collections::HashMap<String, String>) -> Environment {
    with_custom_variables(
        Environment {
            id: String::from("production"),
            variables: collections::HashMap::from([(
                ffi::OsString::from("KEREK_SSH_CONFIGURATION"),
                ["safe", "ssh_configuration"]
                    .iter()
                    .collect::<path::PathBuf>()
                    .into(),
            )]),
        },
        custom_variables,
    )
}
