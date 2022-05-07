use crate::library::constants;
use crate::library::run_command;
use std::fs;
use std::path;
use std::process;

pub fn go() -> anyhow::Result<()> {
    remove_vm_if_exists()?;
    remove_work_folder_if_exists()
}

fn remove_vm_if_exists() -> anyhow::Result<()> {
    let vagrantfile = [constants::WORK_FOLDER, "Vagrantfile"]
        .iter()
        .collect::<path::PathBuf>();

    if vagrantfile.exists() {
        run_command::go(
            process::Command::new("vagrant")
                .args(["destroy", "--force"])
                .current_dir(constants::WORK_FOLDER),
        )
    } else {
        Ok(())
    }
}

fn remove_work_folder_if_exists() -> anyhow::Result<()> {
    if path::Path::new(constants::WORK_FOLDER).exists() {
        fs::remove_dir_all(constants::WORK_FOLDER)?;
    }
    Ok(())
}
