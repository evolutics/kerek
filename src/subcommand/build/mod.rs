use crate::library::command;
use crate::library::compose;
use std::path;
use std::process;

pub fn go(in_: In) -> anyhow::Result<()> {
    let project = compose::parse(compose::Parameters {
        compose_file: &in_.compose_file,
        project_folder: in_.project_folder,
        project_name: in_.project_name,
    })?;

    let project_name = project.name;
    for (key, service) in project.services {
        let image_name = format!("{project_name}-{key}");
        build_image(&image_name, &service)?;
    }

    Ok(())
}

pub struct In {
    pub compose_file: String,
    pub project_folder: Option<String>,
    pub project_name: Option<String>,
}

fn build_image(image_name: &str, service: &compose::Service) -> anyhow::Result<()> {
    let build_context = path::Path::new(&service.build);
    eprintln!("Building image {image_name:?} for context {build_context:?}.");
    command::status_ok(
        process::Command::new("podman")
            .args(["build", "--tag", image_name, "--"])
            .arg(build_context)
            .stderr(process::Stdio::inherit()),
    )
}
