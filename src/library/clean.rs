use super::constants;
use super::run_command;
use std::fs;
use std::path;
use std::process;

pub fn go(work_folder: &path::Path) -> anyhow::Result<()> {
    remove_vm_if_exists(work_folder)?;
    remove_work_folder_if_exists(work_folder)
}

fn remove_vm_if_exists(work_folder: &path::Path) -> anyhow::Result<()> {
    if work_folder.join(constants::VAGRANTFILE_FILENAME).exists() {
        run_command::go(
            process::Command::new("vagrant")
                .arg("destroy")
                .arg("--force")
                .current_dir(work_folder),
        )
    } else {
        Ok(())
    }
}

fn remove_work_folder_if_exists(work_folder: &path::Path) -> anyhow::Result<()> {
    if work_folder.exists() {
        fs::remove_dir_all(work_folder)?;
    }
    Ok(())
}
