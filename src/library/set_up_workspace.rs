use super::assets;
use anyhow::Context;
use std::fs;
use std::path;

pub fn go(work_folder: &path::Path) -> anyhow::Result<()> {
    fs::create_dir_all(work_folder)
        .with_context(|| format!("Unable to create folder: {work_folder:?}"))?;
    for (filename, contents) in [
        (assets::PROVISION_FILENAME, assets::PROVISION),
        (assets::VAGRANTFILE_FILENAME, assets::VAGRANTFILE),
    ] {
        fs::write(work_folder.join(filename), contents)?;
    }
    Ok(())
}
