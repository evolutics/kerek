use super::configuration;
use anyhow::Context;
use std::fs;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    let folder = &configuration.cache.folder;
    fs::create_dir_all(folder).with_context(|| format!("Unable to create folder: {folder:?}"))?;

    for (file, contents) in [
        (
            &configuration.cache.provision,
            include_str!("assets/provision.sh"),
        ),
        (
            &configuration.cache.vagrantfile,
            &format!(
                include_str!("assets/Vagrantfile"),
                ip_address = &configuration.staging.ip_address,
            ),
        ),
    ] {
        fs::write(file, contents)?;
    }

    Ok(())
}
