use super::configuration;
use anyhow::Context;
use std::fs;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    for folder in [&configuration.cache.folder, &configuration.cache.workbench] {
        fs::create_dir_all(folder)
            .with_context(|| format!("Unable to create folder: {folder:?}"))?;
    }

    for (file, contents) in [
        (&configuration.cache.build, include_str!("assets/build.py")),
        (
            &configuration.cache.deploy,
            include_str!("assets/deploy.py"),
        ),
        (
            &configuration.cache.deploy_on_remote,
            include_str!("assets/deploy_on_remote.py"),
        ),
        (
            &configuration.cache.move_to_next_version,
            include_str!("assets/move_to_next_version.sh"),
        ),
        (
            &configuration.cache.provision,
            include_str!("assets/provision.py"),
        ),
        (
            &configuration.cache.provision_on_remote,
            include_str!("assets/provision_on_remote.sh"),
        ),
        (
            &configuration.cache.vagrantfile,
            include_str!("assets/Vagrantfile"),
        ),
    ] {
        fs::write(file, contents)?;
    }

    Ok(())
}
