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
            file,
            project_directory,
            project_name,
        } = &self.compose_arguments;

        for file in file {
            command.args(["--file", file]);
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
