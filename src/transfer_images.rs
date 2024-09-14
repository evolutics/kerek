use super::command;
use super::docker;
use super::log;
use anyhow::Context;
use std::io;

pub fn go(
    In {
        docker_cli,
        dry_run,
        images,
    }: In,
) -> anyhow::Result<()> {
    let images = get_images(images)?;

    for image in images {
        if dry_run {
            log::info!("Would transfer image {image:?}.");
        } else {
            log::info!("Transferring image {image:?}.");

            command::piped_ok(
                docker_cli
                    .default_daemon()
                    .command()
                    .args(["save", "--", &image]),
                docker_cli.command().arg("load"),
            )
            .with_context(|| format!("Unable to transfer image {image:?}"))?;
        }
    }

    Ok(())
}

pub struct In<'a> {
    pub docker_cli: docker::Cli<'a>,
    pub dry_run: bool,
    pub images: Vec<String>,
}

fn get_images(mut images: Vec<String>) -> anyhow::Result<Vec<String>> {
    // TODO: Offer option to only transfer images not already on host.
    Ok(match images.iter().position(|image| image == "-") {
        None => images,
        Some(stdin_index) => {
            let stdin_images = io::stdin()
                .lines()
                .collect::<io::Result<Vec<_>>>()
                .context("Unable to read stdin lines")?;
            images.splice(stdin_index..=stdin_index, stdin_images);
            images
        }
    })
}
