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
    pub build_contexts: Vec<String>,
    pub deploy_user: String,
    pub remote_images_folder: String,
    pub workbench: path::PathBuf,
}

impl Default for Wheelsticks {
    fn default() -> Self {
        Self {
            build_contexts: vec![],
            deploy_user: "wheelsticks".to_owned(),
            remote_images_folder: "images".to_owned(),
            workbench: ".wheelsticks".into(),
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
                    build_contexts: vec![
                        "my_build_context_0".to_owned(),
                        "my_build_context_1".to_owned(),
                    ],
                    deploy_user: "my_deploy_user".to_owned(),
                    remote_images_folder: "my_remote_images_folder".to_owned(),
                    workbench: "my_workbench".into(),
                },
            },
        );
        Ok(())
    }
}
