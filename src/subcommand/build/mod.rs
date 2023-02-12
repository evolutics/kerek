use crate::library::command;
use crate::library::configuration;
use std::collections;
use std::fs;
use std::path;
use std::process;

pub fn go(configuration: path::PathBuf) -> anyhow::Result<()> {
    let configuration = configuration::get(&configuration)?;

    // TODO: Short-circuit if building to deploy on same machine without SSH.

    let build_contexts = configuration.x_wheelsticks.build_contexts;

    let local_workbench = configuration.x_wheelsticks.local_workbench;
    fs::create_dir_all(&local_workbench)?;

    let image_files = build_contexts
        .iter()
        .flat_map(|build_context| build_image_file(build_context, &local_workbench))
        .collect::<collections::BTreeSet<_>>();

    let existing_files = fs::read_dir(local_workbench)?
        .into_iter()
        .flatten()
        .map(|entry| entry.path())
        .collect::<collections::BTreeSet<_>>();

    let obsolete_files = existing_files.difference(&image_files);
    for obsolete_file in obsolete_files {
        println!("Removing obsolete file {obsolete_file:?}.");
        fs::remove_file(obsolete_file)?;
    }

    Ok(())
}

fn build_image_file(
    build_context: &path::Path,
    local_workbench: &path::Path,
) -> anyhow::Result<path::PathBuf> {
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
