use super::configuration;
use anyhow::Context;
use std::fs;

pub fn go(workspace: &configuration::WorkspaceConfiguration) -> anyhow::Result<()> {
    let folder = &workspace.folder;
    fs::create_dir_all(folder).with_context(|| format!("Unable to create folder: {folder:?}"))?;
    for (file, contents) in [
        (
            &workspace.move_to_next_version,
            include_str!("assets/move_to_next_version.sh"),
        ),
        (&workspace.provision, include_str!("assets/provision.sh")),
        (&workspace.vagrantfile, include_str!("assets/Vagrantfile")),
    ] {
        fs::write(file, contents)?;
    }
    Ok(())
}
