use std::process;

pub struct Cli {
    compose_arguments: ComposeArguments,
    compose_engine: Vec<String>,
    container_engine: String,
    docker_arguments: DockerArguments,
}

pub struct ComposeArguments {
    pub ansi: Option<String>,
    pub compatibility: bool,
    pub env_file: Vec<String>,
    pub file: Vec<String>,
    pub parallel: Option<String>,
    pub profile: Vec<String>,
    pub progress: Option<String>,
    pub project_directory: Option<String>,
    pub project_name: Option<String>,
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

impl Cli {
    pub fn new(
        docker_arguments: DockerArguments,
        compose_arguments: ComposeArguments,
        container_engine: String,
        compose_engine: Vec<String>,
    ) -> Self {
        Cli {
            compose_arguments,
            compose_engine,
            container_engine,
            docker_arguments,
        }
    }

    pub fn docker(&self) -> process::Command {
        self.with_docker_arguments(process::Command::new(&self.container_engine))
    }

    fn with_docker_arguments(&self, mut command: process::Command) -> process::Command {
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
            .args(context.iter().flat_map(|context| ["--context", context]))
            .args(["--debug"].iter().filter(|_| *debug))
            .args(host.iter().flat_map(|host| ["--host", host]))
            .args(
                log_level
                    .iter()
                    .flat_map(|log_level| ["--log-level", log_level]),
            )
            .args(["--tls"].iter().filter(|_| *tls))
            .args(
                tlscacert
                    .iter()
                    .flat_map(|tlscacert| ["--tlscacert", tlscacert]),
            )
            .args(tlscert.iter().flat_map(|tlscert| ["--tlscert", tlscert]))
            .args(tlskey.iter().flat_map(|tlskey| ["--tlskey", tlskey]))
            .args(["--tlsverify"].iter().filter(|_| *tlsverify));

        command
    }

    pub fn docker_compose(&self) -> process::Command {
        let mut command = process::Command::new(&self.compose_engine[0]);
        command = self.with_docker_arguments(command);
        command.args(&self.compose_engine[1..]);

        let ComposeArguments {
            ansi,
            compatibility,
            env_file,
            file,
            parallel,
            profile,
            progress,
            project_directory,
            project_name,
        } = &self.compose_arguments;

        command
            .args(ansi.iter().flat_map(|ansi| ["--ansi", ansi]))
            .args(["--compatibility"].iter().filter(|_| *compatibility))
            .args(
                env_file
                    .iter()
                    .flat_map(|env_file| ["--env-file", env_file]),
            )
            .args(file.iter().flat_map(|file| ["--file", file]))
            .args(
                parallel
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
