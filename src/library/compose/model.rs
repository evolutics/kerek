use std::collections;
use std::path;

#[derive(Debug, Default, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Project {
    pub name: Option<String>,

    pub services: collections::BTreeMap<String, Service>,

    #[serde(default, rename = "x-wheelsticks")]
    pub x_wheelsticks: Wheelsticks,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Service {
    pub build: path::PathBuf,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Wheelsticks {
    pub local_workbench: path::PathBuf,
    pub remote_workbench: path::PathBuf,
}

impl Default for Wheelsticks {
    fn default() -> Self {
        Self {
            local_workbench: ".wheelsticks".into(),
            remote_workbench: ".wheelsticks".into(),
        }
    }
}
