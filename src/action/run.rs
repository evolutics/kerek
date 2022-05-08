use crate::library::clean;
use crate::library::configuration;
use crate::library::constants;
use crate::library::loop_until_sigint;
use anyhow::Context;
use std::fs;

pub fn go() -> anyhow::Result<()> {
    let configuration = configuration::get()?;

    loop_until_sigint::go(
        set_up_work_folder,
        || iterate(&configuration),
        || clean::go().expect("Unable to clean."),
    )
}

fn set_up_work_folder() -> anyhow::Result<()> {
    let path = constants::WORK_FOLDER;
    fs::create_dir(path)
        .with_context(|| format!("Unable to create folder, consider cleaning: {path}"))?;
    fs::write(constants::vagrantfile(), include_str!("Vagrantfile"))?;
    Ok(())
}

fn iterate(configuration: &configuration::Data) -> anyhow::Result<()> {
    eprintln!("{configuration:#?}");
    Ok(())
}
