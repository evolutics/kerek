use super::constants;
use std::fs;
use std::io;

pub fn get() -> Result<Data, String> {
    let file = fs::File::open(constants::CONFIGURATION_FILE).map_err(|error| format!("{error}"))?;
    serde_json::from_reader(io::BufReader::new(file)).map_err(|error| format!("{error}"))
}

#[derive(Debug, serde::Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Data {
    #[serde(default = "default_test_base")]
    pub test_base: String,
    #[serde(default = "default_test_staging")]
    pub test_staging: String,
    #[serde(default = "default_test_production")]
    pub test_production: String,

    #[serde(default = "default_production_kubeconfig")]
    pub production_kubeconfig: String,
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
