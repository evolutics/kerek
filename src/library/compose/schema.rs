use std::collections;
use std::path;

#[derive(serde::Deserialize)]
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

#[derive(Default, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Wheelsticks {
    pub local_workbench: Option<path::PathBuf>,
    pub remote_workbench: Option<path::PathBuf>,
}
