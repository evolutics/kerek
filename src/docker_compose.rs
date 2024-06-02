use super::docker;
use std::process;

pub struct Cli<'a> {
    arguments: Arguments<'a>,
    docker_cli: docker::Cli<'a>,
}

pub struct Arguments<'a> {
    pub ansi: Option<&'a str>,
    pub compatibility: bool,
    pub env_file: &'a [String],
    pub file: &'a [String],
    pub parallel: Option<i16>,
    pub profile: &'a [String],
    pub progress: Option<&'a str>,
    pub project_directory: Option<&'a str>,
    pub project_name: Option<&'a str>,
}

impl<'a> Cli<'a> {
    pub fn new(
        docker_arguments: docker::Arguments<'a>,
        docker_compose_arguments: Arguments<'a>,
    ) -> Self {
        Self {
            arguments: docker_compose_arguments,
            docker_cli: docker::Cli::new("docker", docker_arguments),
        }
    }

    pub fn command(&self) -> process::Command {
        let mut command = self.docker_cli.command();

        let Arguments {
            ansi,
            compatibility,
            env_file,
            file,
            parallel,
            profile,
            progress,
            project_directory,
            project_name,
        } = &self.arguments;

        command
            .arg("compose")
            .args(ansi.iter().flat_map(|ansi| ["--ansi", ansi]))
            .args(compatibility.then_some("--compatibility").iter())
            .args(
                env_file
                    .iter()
                    .flat_map(|env_file| ["--env-file", env_file]),
            )
            .args(file.iter().flat_map(|file| ["--file", file]))
            .args(
                parallel
                    .map(|parallel| parallel.to_string())
                    .iter()
                    .flat_map(|parallel| ["--parallel", parallel]),
            )
            .args(profile.iter().flat_map(|profile| ["--profile", profile]))
            .args(
                progress
                    .iter()
                    .flat_map(|progress| ["--progress", progress]),
            )
            .args(
                project_directory
                    .iter()
                    .flat_map(|project_directory| ["--project-directory", project_directory]),
            )
            .args(
                project_name
                    .iter()
                    .flat_map(|project_name| ["--project-name", project_name]),
            );

        command
    }
}
