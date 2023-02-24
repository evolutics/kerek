use super::interpolated;
use std::collections;
use String as StrBuf;

pub const ALIEN_FIELD_MARK: &str = "x-wheelsticks-alien";

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Project {
    pub name: Option<interpolated::StrBuf>,

    pub services: collections::BTreeMap<StrBuf, Service>,

    #[serde(default, rename = "x-wheelsticks")]
    pub x_wheelsticks: Wheelsticks,

    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

#[derive(serde::Deserialize, serde::Serialize)]
pub struct Service {
    pub build: interpolated::StrBuf,
    pub profiles: UnsupportedField,
    #[serde(flatten)]
    pub unknown_fields: UnknownFields,
}

#[derive(Default, serde::Deserialize, serde::Serialize)]
pub struct Wheelsticks {
    pub local_workbench: Option<interpolated::StrBuf>,
    pub remote_workbench: Option<interpolated::StrBuf>,
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

pub type UnknownFields = collections::BTreeMap<StrBuf, Unknown>;

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
