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
    Ok(promote(project))
}

fn promote(project: schema::Project) -> ir::Project {
    ir::Project {
        // TODO: Follow Compose specification for project name.
        name: project.name.unwrap_or_default(),
        services: project.services,
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
            },
        );
        Ok(())
    }
}
