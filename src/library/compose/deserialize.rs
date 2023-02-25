use anyhow::Context;
use serde::de;
use std::path;

pub fn go<T: de::DeserializeOwned>(file: &path::Path, contents: &str) -> anyhow::Result<T> {
    match file.extension() {
        Some(extension) if extension == "toml" => toml::from_str(contents)
            .with_context(|| format!("Unable to deserialize TOML Compose file {file:?}")),
        _ => serde_yaml::from_str(contents)
            .with_context(|| format!("Unable to deserialize YAML Compose file {file:?}")),
    }
}
