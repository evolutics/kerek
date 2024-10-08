use super::command;
use super::docker;
use super::log;
use anyhow::Context;
use std::collections::hash_set;
use std::io;
use std::process;

pub fn go(
    In {
        compress,
        docker_cli,
        dry_run,
        force,
        images,
    }: In,
) -> anyhow::Result<()> {
    let source_docker_cli = docker_cli.default_daemon();
    let destination_docker_cli = docker_cli;

    let images_on_destination = if force {
        Default::default()
    } else {
        get_available_images(&destination_docker_cli)
            .context("Unable to check available images on destination")?
    };

    for image in get_requested_images(images)? {
        if images_on_destination.contains(&image) {
            log::info!("Skipping image {image:?} as found on destination.");
        } else if dry_run {
            log::info!("Would transfer image {image:?}.");
        } else {
            log::info!("Transferring image {image:?}.");

            let mut save = source_docker_cli.command();
            save.args(["save", "--", &image]);
            let mut compress = optional_command(&compress);
            let mut load = destination_docker_cli.command();
            load.arg("load");

            command::piped_ok(
                [Some(&mut save), compress.as_mut(), Some(&mut load)]
                    .into_iter()
                    .flatten(),
            )
            .with_context(|| format!("Unable to transfer image {image:?}"))?;
        }
    }

    Ok(())
}

pub struct In<'a> {
    pub compress: Vec<String>,
    pub docker_cli: docker::Cli<'a>,
    pub dry_run: bool,
    pub force: bool,
    pub images: Vec<String>,
}

fn get_available_images(docker_cli: &docker::Cli) -> anyhow::Result<hash_set::HashSet<String>> {
    Ok(command::stdout_utf8(docker_cli.command().args([
        "images",
        "--format",
        "{{.Repository}}:{{.Tag}}
{{.Repository}}@{{.Digest}}
{{.Repository}}:{{.Tag}}@{{.Digest}}",
    ]))?
    .lines()
    .map(|image| image.into())
    .collect())
}

fn get_requested_images(mut argument_images: Vec<String>) -> anyhow::Result<Vec<String>> {
    Ok(
        match argument_images.iter().position(|image| image == "-") {
            None => argument_images,
            Some(stdin_index) => {
                let stdin_images = io::stdin()
                    .lines()
                    .collect::<io::Result<Vec<_>>>()
                    .context("Unable to read stdin lines")?;
                argument_images.splice(stdin_index..=stdin_index, stdin_images);
                argument_images
            }
        },
    )
}

fn optional_command(command: &[String]) -> Option<process::Command> {
    command.split_first().map(|(program, arguments)| {
        let mut command = process::Command::new(program);
        command.args(arguments);
        command
    })
}
