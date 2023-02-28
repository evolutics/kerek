use std::collections;

pub const ALIEN_FIELD_MARK: &str = "x-wheelsticks-alien";

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
    pub build: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub profiles: UnsupportedField,
    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Wheelsticks {
    pub local_workbench: Option<String>,
    pub remote_workbench: Option<String>,
    #[serde(default)]
    pub schema_mode: SchemaMode,
    pub systemd_unit_folder: Option<String>,
    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

#[derive(Clone, Debug, Default, PartialEq, serde::Deserialize, serde::Serialize)]
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
        mark_alien_field("← unknown").serialize(serializer)
    }
}

impl serde::Serialize for Unsupported {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        mark_alien_field("← unsupported").serialize(serializer)
    }
}

fn mark_alien_field<'a, T>(comment: T) -> collections::HashMap<&'a str, T> {
    [(ALIEN_FIELD_MARK, comment)].into()
}
