use super::configuration;
use anyhow::Context;
use std::fs;

pub fn go(cache: &configuration::Cache) -> anyhow::Result<()> {
    let folder = &cache.folder;
    fs::create_dir_all(folder).with_context(|| format!("Unable to create folder: {folder:?}"))?;
    for (file, contents) in [
        (&cache.provision, include_str!("assets/provision.sh")),
        (&cache.vagrantfile, include_str!("assets/Vagrantfile")),
    ] {
        fs::write(file, contents)?;
    }
    Ok(())
}
