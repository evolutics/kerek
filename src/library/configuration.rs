use super::constants;
use anyhow::Context;
use std::fs;
use std::io;

pub fn get() -> anyhow::Result<Data> {
    let path = constants::CONFIGURATION_FILE;
    let file = fs::File::open(path).with_context(|| format!("Unable to open file: {path}"))?;
    Ok(serde_json::from_reader(io::BufReader::new(file))?)
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    #[serde(default = "default_provision_extras")]
    pub provision_extras: String,

    #[serde(default = "default_test_base")]
    pub test_base: String,
    #[serde(default = "default_test_staging")]
    pub test_staging: String,
    #[serde(default = "default_test_production")]
    pub test_production: String,

    #[serde(default = "default_production_kubeconfig")]
    pub production_kubeconfig: String,
}

fn default_provision_extras() -> String {
    String::from("scripts/provision_extras.sh")
}

fn default_test_base() -> String {
    String::from("scripts/test_base.sh")
}

fn default_test_staging() -> String {
    String::from("scripts/test_staging.sh")
}

fn default_test_production() -> String {
    String::from("scripts/test_production.sh")
}

fn default_production_kubeconfig() -> String {
    String::from("safe/production_kubeconfig")
}
