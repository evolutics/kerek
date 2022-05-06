use std::path;

pub const WORK_FOLDER: &str = ".kerek";

pub fn vagrantfile() -> path::PathBuf {
    [WORK_FOLDER, "Vagrantfile"].iter().collect()
}
