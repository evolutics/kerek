use crate::library::compose;
use std::path;

pub fn go(in_: In) -> anyhow::Result<()> {
    let project = compose::parse(compose::Parameters {
        compose_file: &in_.compose_file,
        environment_files: in_.environment_files,
        project_folder: in_.project_folder,
        project_name: in_.project_name,
    })?;
    print!("{}", compose::print(project)?);

    Ok(())
}

pub struct In {
    pub compose_file: path::PathBuf,
    pub environment_files: Option<Vec<path::PathBuf>>,
    pub project_folder: Option<path::PathBuf>,
    pub project_name: Option<String>,
}
