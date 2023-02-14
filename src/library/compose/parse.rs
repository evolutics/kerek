use super::model;
use anyhow::Context;
use std::fs;
use std::io;
use std::path;

pub fn go(path: &path::Path) -> anyhow::Result<model::Main> {
    let file =
        fs::File::open(path).with_context(|| format!("Unable to open Compose file {path:?}"))?;
    serde_yaml::from_reader(io::BufReader::new(file))
        .with_context(|| format!("Unable to deserialize Compose file {path:?}"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_minimal() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("test_minimal.yaml"))?;

        let main = go(file.as_ref())?;

        assert_eq!(main, model::Main::default());
        Ok(())
    }

    #[test]
    fn handles_full() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("test_full.yaml"))?;

        let main = go(file.as_ref())?;

        assert_eq!(
            main,
            model::Main {
                x_wheelsticks: model::Wheelsticks {
                    build_contexts: vec!["my_build_context_0".into(), "my_build_context_1".into()],
                    local_workbench: "my_local_workbench".into(),
                    remote_workbench: "my_remote_workbench".into(),
                },
            },
        );
        Ok(())
    }
}
