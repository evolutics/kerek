use anyhow::Context;
use std::collections;
use std::ffi;
use std::fs;
use std::io;
use std::path;

pub fn get(path: path::PathBuf) -> anyhow::Result<Main> {
    let file = fs::File::open(&path)
        .with_context(|| format!("Unable to open configuration file: {path:?}"))?;
    let main = serde_json::from_reader(io::BufReader::new(file))
        .with_context(|| format!("Unable to deserialize configuration file: {path:?}"))?;
    Ok(convert(main))
}

pub struct Main {
    pub cache: Cache,
    pub vagrantfile: Option<path::PathBuf>,
    pub life_cycle: LifeCycle,
    pub tests: Tests,
    pub variables: collections::HashMap<ffi::OsString, ffi::OsString>,
    pub staging: Environment,
    pub production: Environment,
}

pub struct Cache {
    pub folder: path::PathBuf,
    pub scripts: path::PathBuf,
    pub ssh_configuration: path::PathBuf,
    pub vagrantfile: path::PathBuf,
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
    pub vagrantfile: Option<path::PathBuf>,
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
    let cache = get_cache(main.cache_folder.unwrap_or_else(|| ".kerek".into()));
    let vagrantfile = main.vagrantfile;
    let life_cycle = get_life_cycle(&cache, main.life_cycle);
    let tests = get_tests(&cache, main.tests);
    let variables = get_variables();
    let staging = get_staging(&cache, main.environment_variables.staging);
    let production = get_production(main.environment_variables.production);

    Main {
        cache,
        vagrantfile,
        life_cycle,
        tests,
        variables,
        staging,
        production,
    }
}

fn get_cache(folder: path::PathBuf) -> Cache {
    Cache {
        scripts: folder.join("scripts.sh"),
        ssh_configuration: folder.join("ssh_configuration"),
        vagrantfile: folder.join("Vagrantfile"),
        folder,
    }
}

fn get_life_cycle(cache: &Cache, life_cycle: UserFacingLifeCycle) -> LifeCycle {
    LifeCycle {
        provision: command_or_default(life_cycle.provision, cache, "provision"),
        build: command_or_default(life_cycle.build, cache, "build"),
        deploy: command_or_default(life_cycle.deploy, cache, "deploy"),
        move_to_next_version: command_or_default(
            life_cycle.move_to_next_version,
            cache,
            "move_to_next_version",
        ),
    }
}

fn command_or_default(command: Vec<String>, cache: &Cache, default: &str) -> Vec<ffi::OsString> {
    if command.is_empty() {
        vec![
            "bash".into(),
            "--".into(),
            (&cache.scripts).into(),
            default.into(),
        ]
    } else {
        command.into_iter().map(|element| element.into()).collect()
    }
}

fn get_tests(cache: &Cache, tests: UserFacingTests) -> Tests {
    Tests {
        base: command_or_default(tests.base, cache, "base_test"),
        smoke: command_or_default(tests.smoke, cache, "smoke_test"),
        acceptance: command_or_default(tests.acceptance, cache, "acceptance_test"),
    }
}

fn get_variables() -> collections::HashMap<ffi::OsString, ffi::OsString> {
    [("KEREK_GIT_BRANCH".into(), "origin/main".into())].into()
}

fn get_staging(
    cache: &Cache,
    custom_variables: collections::HashMap<String, String>,
) -> Environment {
    with_custom_variables(
        Environment {
            id: "staging".into(),
            variables: [
                ("KEREK_IP_ADDRESS".into(), "192.168.60.158".into()),
                (
                    "KEREK_SSH_CONFIGURATION".into(),
                    (&cache.ssh_configuration).into(),
                ),
                ("KEREK_SSH_HOST".into(), "staging".into()),
            ]
            .into(),
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
            id: "production".into(),
            variables: [(
                "KEREK_SSH_CONFIGURATION".into(),
                ["safe", "ssh_configuration"]
                    .iter()
                    .collect::<path::PathBuf>()
                    .into(),
            )]
            .into(),
        },
        custom_variables,
    )
}
