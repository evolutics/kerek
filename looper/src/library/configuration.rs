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

pub struct Environment {
    pub id: String,
    pub env_tests: Vec<ffi::OsString>,
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
    pub environments: UserFacingEnvironments,
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
struct UserFacingEnvironments {
    #[serde(default)]
    pub staging: UserFacingEnvironment,
    #[serde(default)]
    pub production: UserFacingEnvironment,
}

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct UserFacingEnvironment {
    #[serde(default)]
    pub env_tests: Vec<String>,
    #[serde(default)]
    pub variables: collections::HashMap<String, String>,
}

fn convert(main: UserFacingMain) -> Main {
    let cache = get_cache(main.cache_folder.unwrap_or_else(|| ".kerek".into()));
    let vagrantfile = main.vagrantfile;
    let life_cycle = get_life_cycle(&cache, main.life_cycle);
    let variables = get_variables(&cache);
    let staging = get_staging(&cache, main.environments.staging);
    let production = get_production(&cache, main.environments.production);

    Main {
        cache,
        vagrantfile,
        life_cycle,
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

fn get_variables(cache: &Cache) -> collections::HashMap<ffi::OsString, ffi::OsString> {
    [
        ("KEREK_CACHE_FOLDER".into(), cache.folder.clone().into()),
        ("KEREK_GIT_BRANCH".into(), "origin/main".into()),
    ]
    .into()
}

fn get_staging(cache: &Cache, environment: UserFacingEnvironment) -> Environment {
    with_custom_variables(
        Environment {
            id: "staging".into(),
            env_tests: command_or_default(environment.env_tests, cache, "staging_env_tests"),
            variables: [
                ("KEREK_IP_ADDRESS".into(), "192.168.60.158".into()),
                (
                    "KEREK_SSH_CONFIGURATION".into(),
                    (&cache.ssh_configuration).into(),
                ),
                ("KEREK_SSH_HOST".into(), "staging".into()),
                ("KEREK_VM_ID".into(), "staging".into()),
            ]
            .into(),
        },
        environment.variables,
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

fn get_production(cache: &Cache, environment: UserFacingEnvironment) -> Environment {
    with_custom_variables(
        Environment {
            id: "production".into(),
            env_tests: command_or_default(environment.env_tests, cache, "production_env_tests"),
            variables: [(
                "KEREK_SSH_CONFIGURATION".into(),
                ["safe", "ssh_configuration"]
                    .iter()
                    .collect::<path::PathBuf>()
                    .into(),
            )]
            .into(),
        },
        environment.variables,
    )
}
