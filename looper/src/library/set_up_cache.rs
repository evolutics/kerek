use super::configuration;
use anyhow::Context;
use std::borrow;
use std::fs;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    let configured_ssh_cache_folder = configuration.cache.folder.join("configured_ssh");

    for folder in [&configuration.cache.folder, &configured_ssh_cache_folder] {
        fs::create_dir_all(folder)
            .with_context(|| format!("Unable to create cache folder: {folder:?}"))?;
    }

    for (file, contents) in [
        (
            &configuration.cache.folder.join("provision_on_remote.sh"),
            include_str!("assets/provision_on_remote.sh"),
        ),
        (
            &configuration.cache.scripts,
            include_str!("assets/scripts.sh"),
        ),
        (
            &configuration.cache.vagrantfile,
            &get_vagrantfile_contents(configuration)?,
        ),
        (
            &configured_ssh_cache_folder.join("ssh"),
            include_str!("assets/ssh.sh"),
        ),
    ] {
        fs::write(file, contents)
            .with_context(|| format!("Unable to write cache file: {file:?}"))?;
    }

    Ok(())
}

fn get_vagrantfile_contents(
    configuration: &configuration::Main,
) -> anyhow::Result<borrow::Cow<str>> {
    Ok(match &configuration.vagrantfile {
        None => include_str!("assets/Vagrantfile").into(),
        Some(path) => {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Unable to read Vagrantfile: {path:?}"))?;
            contents.into()
        }
    })
}
