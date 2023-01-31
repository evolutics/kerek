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

    let images_folder = configuration.x_wheelsticks.workbench;
    fs::create_dir_all(&images_folder)?;

    let image_files = build_contexts
        .iter()
        .flat_map(|build_context| build_image_file(build_context, &images_folder))
        .collect::<collections::BTreeSet<_>>();

    let existing_files = fs::read_dir(images_folder)?
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
    images_folder: &path::Path,
) -> anyhow::Result<path::PathBuf> {
    println!("Building image for context {build_context:?}.");
    let image_id = build_image(build_context)?;

    let image_file = images_folder.join(format!("{image_id}.tar"));

    if !image_file.try_exists()? {
        println!("Saving image {image_id:?} to {image_file:?}.");
        save_image(&image_id, &image_file)?;
    }

    Ok(image_file)
}

fn build_image(build_context: &path::Path) -> anyhow::Result<String> {
    Ok(command::stdout_utf8(
        process::Command::new("podman")
            .arg("build")
            .arg("--quiet")
            .arg("--")
            .arg(build_context)
            .stderr(process::Stdio::inherit()),
    )?
    .trim_end()
    .to_owned())
}

fn save_image(image_id: &str, image_file: &path::Path) -> anyhow::Result<()> {
    command::status_ok(
        process::Command::new("podman")
            .arg("save")
            .arg("--format")
            .arg("oci-archive")
            .arg("--output")
            .arg(image_file)
            .arg("--")
            .arg(image_id),
    )
}
