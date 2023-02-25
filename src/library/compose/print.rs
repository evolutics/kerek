use super::ir;
use super::schema;
use anyhow::Context;

pub fn go(project: ir::Project) -> anyhow::Result<String> {
    let project = demote(project);
    serde_yaml::to_string(&project).context("Unable to serialize Compose file")
}

fn demote(project: ir::Project) -> schema::Project {
    schema::Project {
        name: Some(project.name.into()),
        services: project
            .services
            .into_iter()
            .map(|(key, service)| {
                (
                    key,
                    schema::Service {
                        build: service.build.into(),
                        profiles: None,
                        unknown_fields: [].into(),
                    },
                )
            })
            .collect(),
        x_wheelsticks: schema::Wheelsticks {
            local_workbench: Some(project.x_wheelsticks.local_workbench.into()),
            remote_workbench: Some(project.x_wheelsticks.remote_workbench.into()),
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

    #[test_case::test_case(include_str!("test_minimal_in.yaml"), include_str!("test_minimal_out.yaml"); "minimal")]
    #[test_case::test_case(include_str!("test_maximal_in.yaml"), include_str!("test_maximal_out.yaml"); "maximal")]
    fn handles(input: &str, expected: &str) -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, input)?;
        let project = parse::go(file.as_ref())?;
        let expected =
            serde_yaml::to_string(&serde_yaml::from_str::<serde_yaml::Value>(expected)?)?;

        assert_eq!(go(project)?, expected);
        Ok(())
    }
}
