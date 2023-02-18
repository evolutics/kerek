use std::collections;
use std::path;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Project {
    pub name: Option<String>,

    pub services: collections::BTreeMap<String, Service>,

    #[serde(default, rename = "x-wheelsticks")]
    pub x_wheelsticks: Wheelsticks,

    #[serde(flatten)]
    pub unknowns: Unknowns,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Service {
    pub build: path::PathBuf,
    #[serde(flatten)]
    pub unknowns: Unknowns,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Wheelsticks {
    pub local_workbench: Option<path::PathBuf>,
    pub remote_workbench: Option<path::PathBuf>,
    #[serde(default)]
    pub schema_mode: SchemaMode,
    #[serde(flatten)]
    pub unknowns: Unknowns,
}

#[derive(Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaMode {
    #[default]
    Default,
    Loose,
    Strict,
}

pub type Unknowns = collections::BTreeMap<String, Unknown>;

// This can be anything as long as it and only it is serialized with a YAML tag.
#[derive(serde::Serialize)]
pub enum Unknown {
    Unknown(()),
}

impl<'d> serde::Deserialize<'d> for Unknown {
    fn deserialize<D: serde::Deserializer<'d>>(_deserializer: D) -> Result<Unknown, D::Error> {
        Ok(Unknown::Unknown(()))
    }
}
