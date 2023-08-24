use std::process;

pub struct Cli {
    compose_arguments: ComposeArguments,
    docker_arguments: DockerArguments,
}

pub struct ComposeArguments {
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
    pub fn new(docker_arguments: DockerArguments, compose_arguments: ComposeArguments) -> Self {
        Cli {
            compose_arguments,
            docker_arguments,
        }
    }

    pub fn docker(&self) -> process::Command {
        let mut command = process::Command::new("docker");

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

        if let Some(config) = config {
            command.args(["--config", config]);
        }
        if let Some(context) = context {
            command.args(["--context", context]);
        }
        if *debug {
            command.arg("--debug");
        }
        if let Some(host) = host {
            command.args(["--host", host]);
        }
        if let Some(log_level) = log_level {
            command.args(["--log-level", log_level]);
        }
        if *tls {
            command.arg("--tls");
        }
        if let Some(tlscacert) = tlscacert {
            command.args(["--tlscacert", tlscacert]);
        }
        if let Some(tlscert) = tlscert {
            command.args(["--tlscert", tlscert]);
        }
        if let Some(tlskey) = tlskey {
            command.args(["--tlskey", tlskey]);
        }
        if *tlsverify {
            command.arg("--tlsverify");
        }

        command
    }

    pub fn docker_compose(&self) -> process::Command {
        let mut command = self.docker();
        command.arg("compose");

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

        if let Some(ansi) = ansi {
            command.args(["--ansi", ansi]);
        }
        if *compatibility {
            command.arg("--compatibility");
        }
        for env_file in env_file {
            command.args(["--env-file", env_file]);
        }
        for file in file {
            command.args(["--file", file]);
        }
        if let Some(parallel) = parallel {
            command.args(["--parallel", &parallel.to_string()]);
        }
        for profile in profile {
            command.args(["--profile", profile]);
        }
        if let Some(progress) = progress {
            command.args(["--progress", progress]);
        }
        if let Some(project_directory) = project_directory {
            command.args(["--project-directory", project_directory]);
        }
        if let Some(project_name) = project_name {
            command.args(["--project-name", project_name]);
        }

        command
    }
}
