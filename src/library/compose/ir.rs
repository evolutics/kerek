pub use super::schema::SchemaMode;
use std::collections;

#[derive(Clone, Debug, PartialEq)]
pub struct Project {
    pub name: String,
    pub services: collections::BTreeMap<String, Service>,
    pub x_wheelsticks: Wheelsticks,
    pub alien_fields: Option<serde_yaml::Value>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Service {
    pub build: String,
}

#[derive(Clone, Debug, PartialEq)]
pub struct Wheelsticks {
    pub local_workbench: String,
    pub remote_workbench: String,
    pub schema_mode: SchemaMode,
    pub systemd_unit_folder: String,
}
