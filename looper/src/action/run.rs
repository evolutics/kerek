use crate::library::clean;
use crate::library::configuration;
use crate::library::loop_until_sigint;

pub fn go() -> anyhow::Result<()> {
    let configuration = configuration::get()?;
    loop_until_sigint::go(
        || iterate(&configuration),
        || clean::go().expect("Unable to clean."),
    )
}

fn iterate(configuration: &configuration::Data) -> anyhow::Result<()> {
    eprintln!("{configuration:#?}");
    Ok(())
}
