use super::get_project_name;
use super::interpolated;
use super::ir;
use super::parse_environment_variables;
use super::schema;
use anyhow::Context;
use std::collections;
use std::env;
use std::fs;
use std::io;
use std::iter;
use std::path;

pub fn go(parameters: Parameters) -> anyhow::Result<ir::Project> {
    let file = parameters.compose_file;
    let contents = fs::read_to_string(file)
        .with_context(|| format!("Unable to read Compose file {file:?}"))?;

    let mut variable_overrides = get_variable_overrides(&parameters)?;

    let project_name = get_project_name::go(get_project_name::In {
        compose_contents: &contents,
        compose_file: file,
        override_: parameters.project_name,
        variable_overrides: &variable_overrides,
    });
    variable_overrides.insert("COMPOSE_PROJECT_NAME".into(), Some(project_name.clone()));

    let project = interpolated::deserialize(file, &contents, &variable_overrides)
        .with_context(|| format!("Unable to deserialize Compose file {file:?}"))?;

    let project = promote(project_name, project)?;
    handle_alien_fields(&project)?;

    Ok(project)
}

pub struct Parameters<'a> {
    pub compose_file: &'a path::Path,
    pub environment_files: Option<Vec<String>>,
    pub project_name: Option<String>,
}

const DEFAULT_ENVIRONMENT_FILE: &str = ".env";

const SYMBOLIC_STDIN_PATH: &str = "-";

fn get_variable_overrides(
    parameters: &Parameters,
) -> anyhow::Result<collections::HashMap<String, Option<String>>> {
    let folder = parameters
        .compose_file
        .parent()
        .unwrap_or_else(|| path::Path::new(""));

    let files = match &parameters.environment_files {
        None => [
            atty::isnt(atty::Stream::Stdin).then_some(SYMBOLIC_STDIN_PATH),
            folder
                .join(DEFAULT_ENVIRONMENT_FILE)
                .is_file()
                .then_some(DEFAULT_ENVIRONMENT_FILE),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>(),
        Some(files) => files.iter().map(|file| file.as_ref()).collect(),
    };

    let mut variable_overrides = collections::HashMap::new();

    for file in files {
        let contents = match file {
            SYMBOLIC_STDIN_PATH => io::read_to_string(io::stdin().lock())
                .context("Unable to read stdin as environment file")?,
            _ => {
                let file = folder.join(file);
                fs::read_to_string(&file)
                    .with_context(|| format!("Unable to read environment file {file:?}"))?
            }
        };

        variable_overrides.extend(parse_environment_variables::go(&contents));
    }

    Ok(variable_overrides)
}

fn promote(project_name: String, project: schema::Project) -> anyhow::Result<ir::Project> {
    let value = serde_yaml::to_value(&project)?;
    let user_systemd_folder_original = project
        .x_wheelsticks
        .user_systemd_folder
        .unwrap_or_else(|| ".config/systemd/user".into());

    Ok(ir::Project {
        name: project_name,
        services: project
            .services
            .into_iter()
            .map(|(key, service)| {
                (
                    key,
                    ir::Service {
                        build: service.build,
                    },
                )
            })
            .collect(),
        x_wheelsticks: ir::Wheelsticks {
            local_workbench: project
                .x_wheelsticks
                .local_workbench
                .unwrap_or_else(|| ".wheelsticks".into()),
            remote_workbench: project
                .x_wheelsticks
                .remote_workbench
                .unwrap_or_else(|| ".wheelsticks".into()),
            schema_mode: project.x_wheelsticks.schema_mode,
            user_systemd_folder_absolute: {
                let home =
                    env::var("HOME").context("Unable to fetch `HOME` environment variable")?;
                path::PathBuf::from(home).join(&user_systemd_folder_original)
            },
            user_systemd_folder_original,
        },
        alien_fields: collect_alien_fields(value),
    })
}

fn collect_alien_fields(value: serde_yaml::Value) -> Option<serde_yaml::Value> {
    match value {
        serde_yaml::Value::Mapping(mapping)
            if mapping.keys().eq(iter::once(schema::ALIEN_FIELD_MARK)) =>
        {
            mapping.into_values().next()
        }
        serde_yaml::Value::Mapping(mapping) => {
            let alien_fields = mapping
                .into_iter()
                .flat_map(|(key, value)| match key {
                    serde_yaml::Value::String(key) if key.starts_with("x-") => None,
                    _ => collect_alien_fields(value).map(|alien_fields| (key, alien_fields)),
                })
                .collect::<serde_yaml::Mapping>();
            (!alien_fields.is_empty()).then(|| serde_yaml::Value::Mapping(alien_fields))
        }
        serde_yaml::Value::Sequence(sequence) => {
            let alien_fields = sequence
                .into_iter()
                .flat_map(collect_alien_fields)
                .collect::<Vec<_>>();
            (!alien_fields.is_empty()).then(|| serde_yaml::Value::Sequence(alien_fields))
        }
        _ => None,
    }
}

fn handle_alien_fields(project: &ir::Project) -> anyhow::Result<()> {
    match &project.alien_fields {
        None => Ok(()),
        Some(alien_fields) => {
            let pretty_aliens = serde_yaml::to_string(&alien_fields)?;
            let pretty_aliens = format!("```\n{pretty_aliens}```");

            match project.x_wheelsticks.schema_mode {
                ir::SchemaMode::Default => {
                    eprintln!(
                        "Warning: Compose file has these unrecognized fields, \
                        which are ignored:\n\
                        {pretty_aliens}\n\
                        Use strict mode to turn this into an error."
                    );
                    Ok(())
                }
                ir::SchemaMode::Loose => Ok(()),
                ir::SchemaMode::Strict => Err(anyhow::anyhow!(
                    "Compose file has these unrecognized fields:\n{pretty_aliens}"
                )),
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_empty_string_it_errs() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, "")?;

        assert!(go(Parameters {
            compose_file: file.as_ref(),
            environment_files: Some(vec![]),
            project_name: None,
        })
        .is_err());
        Ok(())
    }

    #[test_case::test_case(include_str!("test_maximal_in.toml"), ".toml"; "TOML")]
    #[test_case::test_case(include_str!("test_maximal_in.yaml"), ".yaml"; "YAML")]
    fn handles_maximal(contents: &str, suffix: &str) -> anyhow::Result<()> {
        let file = tempfile::Builder::new().suffix(suffix).tempfile()?;
        fs::write(&file, contents)?;
        let home = env::var("HOME")?;

        assert_eq!(
            go(Parameters {
                compose_file: file.as_ref(),
                environment_files: Some(vec![]),
                project_name: None,
            })?,
            ir::Project {
                name: "my_project".into(),
                services: [
                    (
                        "my_service_0".into(),
                        ir::Service {
                            build: "my_build_context_0".into(),
                        },
                    ),
                    (
                        "my_service_1".into(),
                        ir::Service {
                            build: "my_build_context_1".into(),
                        },
                    ),
                ]
                .into(),
                x_wheelsticks: ir::Wheelsticks {
                    local_workbench: "my_local_workbench".into(),
                    remote_workbench: "my_remote_workbench".into(),
                    schema_mode: schema::SchemaMode::Loose,
                    user_systemd_folder_absolute: format!("{home}/my_user_systemd_folder").into(),
                    user_systemd_folder_original: "my_user_systemd_folder".into(),
                },
                alien_fields: Some(serde_yaml::from_str(include_str!(
                    "test_alien_fields.yaml"
                ))?),
            },
        );
        Ok(())
    }

    #[test_case::test_case(include_str!("test_minimal_in.toml"), ".toml"; "TOML")]
    #[test_case::test_case(include_str!("test_minimal_in.yaml"), ".yaml"; "YAML")]
    fn handles_minimal(contents: &str, suffix: &str) -> anyhow::Result<()> {
        let file = tempfile::Builder::new().suffix(suffix).tempfile()?;
        fs::write(&file, contents)?;
        let home = env::var("HOME")?;

        assert_eq!(
            go(Parameters {
                compose_file: file.as_ref(),
                environment_files: Some(vec![]),
                project_name: Some("my_project".into()),
            })?,
            ir::Project {
                name: "my_project".into(),
                services: [].into(),
                x_wheelsticks: ir::Wheelsticks {
                    local_workbench: ".wheelsticks".into(),
                    remote_workbench: ".wheelsticks".into(),
                    schema_mode: schema::SchemaMode::Default,
                    user_systemd_folder_absolute: format!("{home}/.config/systemd/user").into(),
                    user_systemd_folder_original: ".config/systemd/user".into(),
                },
                alien_fields: None,
            },
        );
        Ok(())
    }
}
