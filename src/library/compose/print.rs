use super::interpolated;
use super::ir;
use super::schema;
use anyhow::Context;

pub fn go(project: ir::Project) -> anyhow::Result<String> {
    let project = demote(project);
    interpolated::serialize(&project).context("Unable to serialize Compose file")
}

fn demote(project: ir::Project) -> schema::Project {
    schema::Project {
        name: Some(project.name),
        services: project
            .services
            .into_iter()
            .map(|(key, service)| {
                (
                    key,
                    schema::Service {
                        build: service.build,
                        profiles: None,
                        unknown_fields: [].into(),
                    },
                )
            })
            .collect(),
        x_wheelsticks: schema::Wheelsticks {
            local_workbench: Some(project.x_wheelsticks.local_workbench),
            remote_workbench: Some(project.x_wheelsticks.remote_workbench),
            schema_mode: project.x_wheelsticks.schema_mode,
            unknown_fields: [].into(),
        },
        unknown_fields: [].into(),
    }
}

#[cfg(test)]
mod tests {
    use super::super::parse;
    use super::*;
    use std::fs;

    #[test_case::test_case(include_str!("test_maximal_in.yaml"), include_str!("test_maximal_out.yaml"); "maximal")]
    #[test_case::test_case(include_str!("test_minimal_in.yaml"), include_str!("test_minimal_out.yaml"); "minimal")]
    fn handles(input: &str, expected: &str) -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, input)?;
        let project = parse::go(parse::Parameters {
            compose_file: file.as_ref(),
            project_name: Some("my_project".into()),
        })?;

        assert_eq!(go(project)?, expected);
        Ok(())
    }
}
