pub use super::schema::SchemaMode;
use std::collections;

#[derive(Debug, PartialEq)]
pub struct Project {
    pub name: String,
    pub services: collections::BTreeMap<String, Service>,
    pub x_wheelsticks: Wheelsticks,
    pub alien_fields: Option<serde_yaml::Value>,
}

#[derive(Debug, PartialEq)]
pub struct Service {
    pub build: String,
}

#[derive(Debug, PartialEq)]
pub struct Wheelsticks {
    pub local_workbench: String,
    pub remote_workbench: String,
    pub schema_mode: SchemaMode,
}
