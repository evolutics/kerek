use super::ir;
use super::schema;
use anyhow::Context;
use std::fs;
use std::io;
use std::path;

pub fn go(path: &path::Path) -> anyhow::Result<ir::Project> {
    let file =
        fs::File::open(path).with_context(|| format!("Unable to open Compose file {path:?}"))?;
    let project = serde_yaml::from_reader(io::BufReader::new(file))
        .with_context(|| format!("Unable to deserialize Compose file {path:?}"))?;
    let project = promote(project)?;
    handle_unknowns(&project)?;
    Ok(project)
}

fn promote(project: schema::Project) -> anyhow::Result<ir::Project> {
    let value = serde_yaml::to_value(&project)?;

    Ok(ir::Project {
        // TODO: Follow Compose specification for project name.
        name: project.name.unwrap_or_default(),
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
        },
        unknowns: collect_unknowns(value),
    })
}

fn collect_unknowns(value: serde_yaml::Value) -> Option<serde_yaml::Value> {
    match value {
        serde_yaml::Value::Mapping(mapping) => {
            let unknowns = mapping
                .into_iter()
                .flat_map(|(key, value)| collect_unknowns(value).map(|unknowns| (key, unknowns)))
                .collect::<serde_yaml::Mapping>();
            (!unknowns.is_empty()).then(|| serde_yaml::Value::Mapping(unknowns))
        }
        serde_yaml::Value::Sequence(sequence) => {
            let unknowns = sequence
                .into_iter()
                .flat_map(collect_unknowns)
                .collect::<Vec<_>>();
            (!unknowns.is_empty()).then(|| serde_yaml::Value::Sequence(unknowns))
        }
        serde_yaml::Value::Tagged(_) => Some("← unknown".into()),
        _ => None,
    }
}

fn handle_unknowns(project: &ir::Project) -> anyhow::Result<()> {
    if let Some(unknowns) = &project.unknowns {
        let pretty_unknowns = serde_yaml::to_string(&unknowns)?;
        println!(
            "Warning: Compose file has these unknown fields, \
which are ignored:\n```\n{pretty_unknowns}```"
        );
    }
    Ok(())
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
    fn handles_minimal() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("test_minimal.yaml"))?;

        assert_eq!(
            go(file.as_ref())?,
            ir::Project {
                name: "".into(),
                services: Default::default(),
                x_wheelsticks: ir::Wheelsticks {
                    local_workbench: ".wheelsticks".into(),
                    remote_workbench: ".wheelsticks".into(),
                },
                unknowns: None,
            }
        );
        Ok(())
    }

    #[test]
    fn handles_full() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("test_full.yaml"))?;

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
                },
                unknowns: Some(serde_yaml::from_str(include_str!("test_unknowns.yaml"))?),
            },
        );
        Ok(())
    }
}
