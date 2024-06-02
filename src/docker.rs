use std::process;

pub struct Cli {
    container_engine: String,
    docker_arguments: DockerArguments,
    docker_compose_arguments: DockerComposeArguments,
}

pub struct DockerArguments {
    pub config: Option<String>,
    pub context: Option<String>,
    pub debug: bool,
    pub host: Option<String>,
    pub log_level: Option<String>,
    pub tls: bool,
    pub tlscacert: Option<String>,
    pub tlscert: Option<String>,
    pub tlskey: Option<String>,
    pub tlsverify: bool,
}

pub struct DockerComposeArguments {
    pub ansi: Option<String>,
    pub compatibility: bool,
    pub env_file: Vec<String>,
    pub file: Vec<String>,
    pub parallel: Option<i16>,
    pub profile: Vec<String>,
    pub progress: Option<String>,
    pub project_directory: Option<String>,
    pub project_name: Option<String>,
}

impl Cli {
    pub fn new(
        container_engine: String,
        docker_arguments: DockerArguments,
        docker_compose_arguments: DockerComposeArguments,
    ) -> Self {
        Self {
            container_engine,
            docker_arguments,
            docker_compose_arguments,
        }
    }

    pub fn docker(&self) -> process::Command {
        self.with_docker_arguments(process::Command::new(&self.container_engine), false)
    }

    fn with_docker_arguments(
        &self,
        mut command: process::Command,
        default_daemon: bool,
    ) -> process::Command {
        let DockerArguments {
            config,
            context,
            debug,
            host,
            log_level,
            tls,
            tlscacert,
            tlscert,
            tlskey,
            tlsverify,
        } = &self.docker_arguments;

        command
            .args(config.iter().flat_map(|config| ["--config", config]))
            .args(
                context
                    .iter()
                    .filter(|_| !default_daemon)
                    .flat_map(|context| ["--context", context]),
            )
            .args(debug.then_some("--debug").iter())
            .args(
                host.iter()
                    .filter(|_| !default_daemon)
                    .flat_map(|host| ["--host", host]),
            )
            .args(
                log_level
                    .iter()
                    .flat_map(|log_level| ["--log-level", log_level]),
            )
            .args(tls.then_some("--tls").iter())
            .args(
                tlscacert
                    .iter()
                    .flat_map(|tlscacert| ["--tlscacert", tlscacert]),
            )
            .args(tlscert.iter().flat_map(|tlscert| ["--tlscert", tlscert]))
            .args(tlskey.iter().flat_map(|tlskey| ["--tlskey", tlskey]))
            .args(tlsverify.then_some("--tlsverify").iter());

        command
    }

    pub fn docker_compose(&self) -> process::Command {
        let mut command = self.with_docker_arguments(process::Command::new("docker"), false);

        let DockerComposeArguments {
            ansi,
            compatibility,
            env_file,
            file,
            parallel,
            profile,
            progress,
            project_directory,
            project_name,
        } = &self.docker_compose_arguments;

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

    pub fn docker_default_daemon(&self) -> process::Command {
        self.with_docker_arguments(process::Command::new(&self.container_engine), true)
    }
}
