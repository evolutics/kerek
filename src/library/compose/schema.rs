use serde_yaml::value;
use std::collections;
use std::path;

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Project {
    pub name: Option<String>,

    pub services: collections::BTreeMap<String, Service>,

    #[serde(default, rename = "x-wheelsticks")]
    pub x_wheelsticks: Wheelsticks,

    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Service {
    pub build: path::PathBuf,
    pub profiles: UnsupportedField,
    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Wheelsticks {
    pub local_workbench: Option<path::PathBuf>,
    pub remote_workbench: Option<path::PathBuf>,
    #[serde(default)]
    pub schema_mode: SchemaMode,
    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

#[derive(Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
#[serde(rename_all = "snake_case")]
pub enum SchemaMode {
    #[default]
    Default,
    Loose,
    Strict,
}

pub type UnknownFields = collections::BTreeMap<String, Unknown>;

#[derive(serde::Deserialize)]
pub struct Unknown(serde_yaml::Value);

pub type UnsupportedField = Option<Unsupported>;

#[derive(Default, serde::Deserialize)]
pub struct Unsupported(serde_yaml::Value);

impl serde::Serialize for Unknown {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        value_with_arbitrary_tag("← unknown").serialize(serializer)
    }
}

impl serde::Serialize for Unsupported {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        value_with_arbitrary_tag("← unsupported").serialize(serializer)
    }
}

fn value_with_arbitrary_tag(value: &str) -> value::TaggedValue {
    value::TaggedValue {
        tag: value::Tag::new("Wheelsticks"),
        value: value.into(),
    }
}
