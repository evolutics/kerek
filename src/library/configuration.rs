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
    pub scripts: CacheScripts,
    pub staging: CacheStaging,
    pub workbench: path::PathBuf,
}

pub struct CacheScripts {
    pub folder: path::PathBuf,
    pub build: path::PathBuf,
    pub deploy: path::PathBuf,
    pub deploy_on_remote: path::PathBuf,
    pub move_to_next_version: path::PathBuf,
    pub playbook: path::PathBuf,
    pub provision: path::PathBuf,
    pub provision_test: path::PathBuf,
}

pub struct CacheStaging {
    pub folder: path::PathBuf,
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
    let cache = get_cache(
        main.cache_folder
            .unwrap_or_else(|| path::PathBuf::from(".kerek")),
    );
    let vagrantfile = main.vagrantfile;
    let life_cycle = get_life_cycle(&cache, main.life_cycle);
    let tests = get_tests(main.tests);
    let variables = get_variables(&cache);
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
    let scripts = folder.join("scripts");
    let staging = folder.join("staging");
    let workbench = folder.join("workbench");

    Cache {
        folder,
        scripts: CacheScripts {
            build: scripts.join("build.py"),
            deploy: scripts.join("deploy.py"),
            deploy_on_remote: scripts.join("deploy_on_remote.py"),
            move_to_next_version: scripts.join("move_to_next_version.sh"),
            playbook: scripts.join("playbook.yaml"),
            provision: scripts.join("provision.py"),
            provision_test: scripts.join("provision_test.sh"),
            folder: scripts,
        },
        staging: CacheStaging {
            ssh_configuration: staging.join("ssh_configuration"),
            vagrantfile: staging.join("Vagrantfile"),
            folder: staging,
        },
        workbench,
    }
}

fn get_life_cycle(cache: &Cache, life_cycle: UserFacingLifeCycle) -> LifeCycle {
    LifeCycle {
        provision: convert_nonempty_or_else(life_cycle.provision, || {
            vec![
                "python3".into(),
                "--".into(),
                (&cache.scripts.provision).into(),
            ]
        }),
        build: convert_nonempty_or_else(life_cycle.build, || {
            vec!["python3".into(), "--".into(), (&cache.scripts.build).into()]
        }),
        deploy: convert_nonempty_or_else(life_cycle.deploy, || {
            vec![
                "python3".into(),
                "--".into(),
                (&cache.scripts.deploy).into(),
            ]
        }),
        move_to_next_version: convert_nonempty_or_else(life_cycle.move_to_next_version, || {
            vec![
                "bash".into(),
                "--".into(),
                (&cache.scripts.move_to_next_version).into(),
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
        base: convert_nonempty_or_else(tests.base, || vec!["scripts/base_test.sh".into()]),
        smoke: convert_nonempty_or_else(tests.smoke, || vec!["scripts/smoke_test.sh".into()]),
        acceptance: convert_nonempty_or_else(tests.acceptance, || {
            vec!["scripts/acceptance_test.sh".into()]
        }),
    }
}

fn get_variables(cache: &Cache) -> collections::HashMap<ffi::OsString, ffi::OsString> {
    collections::HashMap::from([
        ("KEREK_CACHE_SCRIPTS".into(), (&cache.scripts.folder).into()),
        ("KEREK_CACHE_WORKBENCH".into(), (&cache.workbench).into()),
        ("KEREK_DEPLOY_USER".into(), "kerek".into()),
        ("KEREK_GIT_BRANCH".into(), "origin/main".into()),
        ("KEREK_MANIFEST_FILE".into(), "images.json".into()),
        ("KEREK_REMOTE_IMAGES_FOLDER".into(), "images".into()),
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
                ("KEREK_IP_ADDRESS".into(), "192.168.60.158".into()),
                (
                    "KEREK_SSH_CONFIGURATION".into(),
                    (&cache.staging.ssh_configuration).into(),
                ),
                ("KEREK_SSH_HOST".into(), "staging".into()),
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
                "KEREK_SSH_CONFIGURATION".into(),
                ["safe", "ssh_configuration"]
                    .iter()
                    .collect::<path::PathBuf>()
                    .into(),
            )]),
        },
        custom_variables,
    )
}
