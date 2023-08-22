use std::process;

pub struct Cli {
    compose_arguments: ComposeArguments,
    docker_arguments: DockerArguments,
}

pub struct ComposeArguments {
    pub file: Vec<String>,
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

        if let Some(config) = &self.docker_arguments.config {
            command.args(["--config", config]);
        }
        if let Some(context) = &self.docker_arguments.context {
            command.args(["--context", context]);
        }
        if self.docker_arguments.debug {
            command.arg("--debug");
        }
        if let Some(host) = &self.docker_arguments.host {
            command.args(["--host", host]);
        }
        if let Some(log_level) = &self.docker_arguments.log_level {
            command.args(["--log-level", log_level]);
        }
        if self.docker_arguments.tls {
            command.arg("--tls");
        }
        if let Some(tlscacert) = &self.docker_arguments.tlscacert {
            command.args(["--tlscacert", tlscacert]);
        }
        if let Some(tlscert) = &self.docker_arguments.tlscert {
            command.args(["--tlscert", tlscert]);
        }
        if let Some(tlskey) = &self.docker_arguments.tlskey {
            command.args(["--tlskey", tlskey]);
        }
        if self.docker_arguments.tlsverify {
            command.arg("--tlsverify");
        }

        command
    }

    pub fn docker_compose(&self) -> process::Command {
        let mut command = self.docker();
        command.arg("compose");

        for file in &self.compose_arguments.file {
            command.args(["--file", file]);
        }
        if let Some(project_directory) = &self.compose_arguments.project_directory {
            command.args(["--project-directory", project_directory]);
        }
        if let Some(project_name) = &self.compose_arguments.project_name {
            command.args(["--project-name", project_name]);
        }

        command
    }
}
