use super::command;
use super::docker;
use super::docker_compose;
use super::log;
use anyhow::Context;

pub fn go(
    In {
        docker_cli,
        docker_compose_cli,
        dry_run,
        images,
    }: In,
) -> anyhow::Result<()> {
    let images = get_images(&docker_compose_cli, images)?;

    for image in images {
        if dry_run {
            log::info!("Would transfer image {image:?}.");
        } else {
            log::info!("Transferring image {image:?}.");

            command::piped_ok(
                docker_cli
                    .command_default_daemon()
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
    pub docker_compose_cli: docker_compose::Cli<'a>,
    pub dry_run: bool,
    pub images: Vec<String>,
}

fn get_images(
    docker_compose_cli: &docker_compose::Cli,
    images: Vec<String>,
) -> anyhow::Result<Vec<String>> {
    Ok(if images.is_empty() {
        // TODO: Select only images not already on host.
        let images =
            command::stdout_utf8(docker_compose_cli.command().args(["config", "--images"]))
                .context("Unable to get images from Compose configuration")?;
        images.lines().map(|image| image.into()).collect()
    } else {
        images
    })
}
