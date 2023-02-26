use crate::library::compose;
use std::path;

pub fn go(in_: In) -> anyhow::Result<()> {
    let project = compose::parse(compose::Parameters {
        compose_file: &in_.compose_file,
        environment_file: None,
        project_name: in_.project_name,
    })?;
    print!("{}", compose::print(project)?);

    Ok(())
}

pub struct In {
    pub compose_file: path::PathBuf,
    pub project_name: Option<String>,
}
