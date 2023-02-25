use super::ir;
use super::schema;
use anyhow::Context;
use std::fs;
use std::io;
use std::iter;
use std::path;

pub fn go(path: &path::Path) -> anyhow::Result<ir::Project> {
    let file =
        fs::File::open(path).with_context(|| format!("Unable to open Compose file {path:?}"))?;
    let project = serde_yaml::from_reader(io::BufReader::new(file))
        .with_context(|| format!("Unable to deserialize Compose file {path:?}"))?;
    let project = promote(project)?;
    handle_alien_fields(&project)?;
    Ok(project)
}

fn promote(project: schema::Project) -> anyhow::Result<ir::Project> {
    let value = serde_yaml::to_value(&project)?;

    Ok(ir::Project {
        // TODO: Follow Compose specification for project name.
        name: project.name.unwrap_or_default().into(),
        services: project
            .services
            .into_iter()
            .map(|(key, service)| {
                (
                    key,
                    ir::Service {
                        build: service.build.into(),
                    },
                )
            })
            .collect(),
        x_wheelsticks: ir::Wheelsticks {
            local_workbench: project
                .x_wheelsticks
                .local_workbench
                .unwrap_or_else(|| ".wheelsticks".into())
                .into(),
            remote_workbench: project
                .x_wheelsticks
                .remote_workbench
                .unwrap_or_else(|| ".wheelsticks".into())
                .into(),
            schema_mode: project.x_wheelsticks.schema_mode,
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
                    println!(
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

        assert!(go(file.as_ref()).is_err());
        Ok(())
    }

    #[test]
    fn handles_maximal() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("test_maximal_in.yaml"))?;

        assert_eq!(
            go(file.as_ref())?,
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
                },
                alien_fields: Some(serde_yaml::from_str(include_str!(
                    "test_alien_fields.yaml"
                ))?),
            },
        );
        Ok(())
    }

    #[test]
    fn handles_minimal() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("test_minimal_in.yaml"))?;

        assert_eq!(
            go(file.as_ref())?,
            ir::Project {
                name: "".into(),
                services: [].into(),
                x_wheelsticks: ir::Wheelsticks {
                    local_workbench: ".wheelsticks".into(),
                    remote_workbench: ".wheelsticks".into(),
                    schema_mode: schema::SchemaMode::Default,
                },
                alien_fields: None,
            }
        );
        Ok(())
    }
}
