pub use super::schema::SchemaMode;
use std::collections;
use std::path;

#[derive(Debug, PartialEq)]
pub struct Project {
    pub name: String,
    pub services: collections::BTreeMap<String, Service>,
    pub x_wheelsticks: Wheelsticks,
    pub unknowns: Option<serde_yaml::Value>,
}

#[derive(Debug, PartialEq)]
pub struct Service {
    pub build: path::PathBuf,
}

#[derive(Debug, PartialEq)]
pub struct Wheelsticks {
    pub local_workbench: path::PathBuf,
    pub remote_workbench: path::PathBuf,
    pub schema_mode: SchemaMode,
}
