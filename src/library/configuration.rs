use anyhow::Context;
use std::fs;
use std::io;
use std::path;

pub fn get(path: &path::Path) -> anyhow::Result<Main> {
    let file = fs::File::open(path)
        .with_context(|| format!("Unable to open configuration file: {path:?}"))?;
    serde_yaml::from_reader(io::BufReader::new(file))
        .with_context(|| format!("Unable to deserialize configuration file: {path:?}"))
}

#[derive(Debug, Default, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Main {
    #[serde(default, rename = "x-wheelsticks")]
    pub x_wheelsticks: Wheelsticks,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Wheelsticks {
    pub build_contexts: Vec<path::PathBuf>,
    pub deploy_user: String,
    pub local_workbench: path::PathBuf,
    pub remote_workbench: path::PathBuf,
}

impl Default for Wheelsticks {
    fn default() -> Self {
        Self {
            build_contexts: vec![],
            deploy_user: "wheelsticks".into(),
            local_workbench: ".wheelsticks".into(),
            remote_workbench: ".wheelsticks".into(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_minimal() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("configuration_test_minimal.yaml"))?;

        let main = get(file.as_ref())?;

        assert_eq!(main, Main::default());
        Ok(())
    }

    #[test]
    fn handles_full() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("configuration_test_full.yaml"))?;

        let main = get(file.as_ref())?;

        assert_eq!(
            main,
            Main {
                x_wheelsticks: Wheelsticks {
                    build_contexts: vec!["my_build_context_0".into(), "my_build_context_1".into()],
                    deploy_user: "my_deploy_user".into(),
                    local_workbench: "my_local_workbench".into(),
                    remote_workbench: "my_remote_workbench".into(),
                },
            },
        );
        Ok(())
    }
}
