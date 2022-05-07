use crate::library::configuration;

pub fn go() -> Result<(), String> {
    let configuration = configuration::get()?;
    eprintln!("{configuration:#?}");
    Ok(())
}
