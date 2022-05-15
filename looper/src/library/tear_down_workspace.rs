use super::command;
use super::configuration;
use std::fs;
use std::process;

pub fn go(workspace: &configuration::WorkspaceConfiguration) -> anyhow::Result<()> {
    remove_vm_if_exists(workspace)?;
    remove_workspace_folder_if_exists(workspace)
}

fn remove_vm_if_exists(workspace: &configuration::WorkspaceConfiguration) -> anyhow::Result<()> {
    if workspace.vagrantfile.exists() {
        command::status(
            process::Command::new("vagrant")
                .arg("destroy")
                .arg("--force")
                .current_dir(&workspace.folder),
        )
    } else {
        Ok(())
    }
}

fn remove_workspace_folder_if_exists(
    workspace: &configuration::WorkspaceConfiguration,
) -> anyhow::Result<()> {
    if workspace.folder.exists() {
        fs::remove_dir_all(&workspace.folder)?;
    }
    Ok(())
}
