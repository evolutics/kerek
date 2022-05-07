use crate::library::configuration;

pub fn go() -> anyhow::Result<()> {
    let configuration = configuration::get()?;
    eprintln!("{configuration:#?}");
    Ok(())
}
