use crate::library::command;
use crate::library::compose;
use std::collections;
use std::fs;
use std::path;
use std::process;

pub fn go(compose_file: path::PathBuf) -> anyhow::Result<()> {
    let project = compose::parse(&compose_file)?;

    // TODO: Short-circuit if building to deploy on same machine without SSH.

    let local_workbench = project.x_wheelsticks.local_workbench;
    fs::create_dir_all(&local_workbench)?;

    let image_files = project
        .services
        .values()
        .flat_map(|service| build_image_file(service, &local_workbench))
        .collect::<collections::BTreeSet<_>>();

    let existing_files = fs::read_dir(local_workbench)?
        .into_iter()
        .map(|result| result.map(|entry| entry.path()))
        .collect::<Result<collections::BTreeSet<_>, _>>()?;

    let obsolete_files = existing_files.difference(&image_files);
    for obsolete_file in obsolete_files {
        println!("Removing obsolete file {obsolete_file:?}.");
        fs::remove_file(obsolete_file)?;
    }

    Ok(())
}

fn build_image_file(
    service: &compose::Service,
    local_workbench: &path::Path,
) -> anyhow::Result<path::PathBuf> {
    let build_context = &service.build;
    println!("Building image for context {build_context:?}.");
    let image_id = build_image(build_context)?;

    let image_file = local_workbench.join(format!("{image_id}.tar"));

    if !image_file.try_exists()? {
        println!("Saving image {image_id:?} to {image_file:?}.");
        save_image(&image_id, &image_file)?;
    }

    Ok(image_file)
}

fn build_image(build_context: &path::Path) -> anyhow::Result<String> {
    Ok(command::stdout_utf8(
        process::Command::new("podman")
            .args(["build", "--quiet", "--"])
            .arg(build_context)
            .stderr(process::Stdio::inherit()),
    )?
    .trim_end()
    .into())
}

fn save_image(image_id: &str, image_file: &path::Path) -> anyhow::Result<()> {
    command::status_ok(
        process::Command::new("podman")
            .args(["save", "--format", "oci-archive", "--output"])
            .arg(image_file)
            .args(["--", image_id]),
    )
}
