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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_minimum() -> anyhow::Result<()> {
        let command = Cli::new(
            docker::Arguments {
                config: None,
                context: None,
                debug: true,
                host: None,
                log_level: None,
                tls: false,
                tlscacert: None,
                tlscert: None,
                tlskey: None,
                tlsverify: false,
            },
            Arguments {
                ansi: None,
                compatibility: false,
                env_file: &[],
                file: &[],
                parallel: None,
                profile: &[],
                progress: None,
                project_directory: None,
                project_name: None,
            },
        )
        .command();

        assert_eq!(command.get_program(), "docker");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["--debug", "compose"],
        );
        Ok(())
    }

    #[test]
    fn handles_maximum() -> anyhow::Result<()> {
        let command = Cli::new(
            docker::Arguments {
                config: None,
                context: None,
                debug: true,
                host: None,
                log_level: None,
                tls: false,
                tlscacert: None,
                tlscert: None,
                tlskey: None,
                tlsverify: false,
            },
            Arguments {
                ansi: Some("ansi"),
                compatibility: true,
                env_file: &["env_file".into()],
                file: &["file".into()],
                parallel: Some(5),
                profile: &["profile".into()],
                progress: Some("progress"),
                project_directory: Some("project_directory"),
                project_name: Some("project_name"),
            },
        )
        .command();

        assert_eq!(command.get_program(), "docker");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                "--debug",
                "compose",
                "--ansi",
                "ansi",
                "--compatibility",
                "--env-file",
                "env_file",
                "--file",
                "file",
                "--parallel",
                "5",
                "--profile",
                "profile",
                "--progress",
                "progress",
                "--project-directory",
                "project_directory",
                "--project-name",
                "project_name",
            ],
        );
        Ok(())
    }
}
