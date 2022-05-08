use std::path;

pub const CONFIGURATION_FILE: &str = "kerek.json";

pub const STAGING_IP: &str = "192.168.63.63";

pub const WORK_FOLDER: &str = ".kerek";

pub fn vagrantfile() -> path::PathBuf {
    [WORK_FOLDER, "Vagrantfile"].iter().collect()
}
