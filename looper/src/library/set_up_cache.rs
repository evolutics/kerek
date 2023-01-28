use super::configuration;
use anyhow::Context;
use std::borrow;
use std::fs;

pub fn go(configuration: &configuration::Main) -> anyhow::Result<()> {
    for folder in [
        &configuration.cache.scripts.folder,
        &configuration.cache.staging.folder,
        &configuration.cache.workbench,
    ] {
        fs::create_dir_all(folder)
            .with_context(|| format!("Unable to create cache folder: {folder:?}"))?;
    }

    for (file, contents) in [
        (
            &configuration.cache.scripts.move_to_next_version,
            include_str!("assets/move_to_next_version.sh"),
        ),
        (
            &configuration.cache.staging.vagrantfile,
            &get_vagrantfile_contents(configuration)?,
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
        None => borrow::Cow::from(include_str!("assets/Vagrantfile")),
        Some(path) => {
            let contents = fs::read_to_string(path)
                .with_context(|| format!("Unable to read Vagrantfile: {path:?}"))?;
            borrow::Cow::from(contents)
        }
    })
}
