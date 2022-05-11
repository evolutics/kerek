use std::path;

pub const CONFIGURATION_FILE: &str = "kerek.json";

pub const PROVISION_BASE: &str = include_str!("provision_base.sh");

pub const VAGRANTFILE: &str = include_str!("Vagrantfile");

pub const WORK_FOLDER: &str = ".kerek";

pub fn provision_base_file() -> path::PathBuf {
    [WORK_FOLDER, "provision_base.sh"].into_iter().collect()
}

pub fn vagrantfile_file() -> path::PathBuf {
    [WORK_FOLDER, "Vagrantfile"].into_iter().collect()
}
