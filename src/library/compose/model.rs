use std::path;

#[derive(Debug, Default, PartialEq, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Main {
    #[serde(default, rename = "x-wheelsticks")]
    pub x_wheelsticks: Wheelsticks,
}

#[derive(Debug, PartialEq, serde::Deserialize)]
#[serde(default, deny_unknown_fields)]
pub struct Wheelsticks {
    pub build_contexts: Vec<path::PathBuf>,
    pub local_workbench: path::PathBuf,
    pub remote_workbench: path::PathBuf,
}

impl Default for Wheelsticks {
    fn default() -> Self {
        Self {
            build_contexts: vec![],
            local_workbench: ".wheelsticks".into(),
            remote_workbench: ".wheelsticks".into(),
        }
    }
}
