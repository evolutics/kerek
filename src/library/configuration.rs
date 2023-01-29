use anyhow::Context;
use std::fs;
use std::io;
use std::path;

#[allow(dead_code)]
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

#[derive(Debug, Default, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wheelsticks {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles() -> anyhow::Result<()> {
        let file = tempfile::NamedTempFile::new()?;
        fs::write(&file, include_str!("configuration_test.yaml"))?;

        let main = get(file.as_ref())?;

        assert_eq!(
            main,
            Main {
                x_wheelsticks: Wheelsticks {},
            },
        );
        Ok(())
    }
}
