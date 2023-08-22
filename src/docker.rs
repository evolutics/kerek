use std::process;

pub struct Cli {
    config: Option<String>,
    context: Option<String>,
    debug: bool,
    file: Vec<String>,
    host: Option<String>,
    log_level: Option<String>,
    project_directory: Option<String>,
    project_name: Option<String>,
    tls: bool,
    tlscacert: Option<String>,
    tlscert: Option<String>,
    tlskey: Option<String>,
    tlsverify: bool,
}

pub struct In {
    pub config: Option<String>,
    pub context: Option<String>,
    pub debug: bool,
    pub file: Vec<String>,
    pub host: Option<String>,
    pub log_level: Option<String>,
    pub project_directory: Option<String>,
    pub project_name: Option<String>,
    pub tls: bool,
    pub tlscacert: Option<String>,
    pub tlscert: Option<String>,
    pub tlskey: Option<String>,
    pub tlsverify: bool,
}

impl Cli {
    pub fn new(
        In {
            config,
            context,
            debug,
            file,
            host,
            log_level,
            project_directory,
            project_name,
            tls,
            tlscacert,
            tlscert,
            tlskey,
            tlsverify,
        }: In,
    ) -> Self {
        Cli {
            config,
            context,
            debug,
            file,
            host,
            log_level,
            project_directory,
            project_name,
            tls,
            tlscacert,
            tlscert,
            tlskey,
            tlsverify,
        }
    }

    pub fn docker(&self) -> process::Command {
        let mut command = process::Command::new("docker");

        if let Some(config) = &self.config {
            command.args(["--config", config]);
        }
        if let Some(context) = &self.context {
            command.args(["--context", context]);
        }
        if self.debug {
            command.arg("--debug");
        }
        if let Some(host) = &self.host {
            command.args(["--host", host]);
        }
        if let Some(log_level) = &self.log_level {
            command.args(["--log-level", log_level]);
        }
        if self.tls {
            command.arg("--tls");
        }
        if let Some(tlscacert) = &self.tlscacert {
            command.args(["--tlscacert", tlscacert]);
        }
        if let Some(tlscert) = &self.tlscert {
            command.args(["--tlscert", tlscert]);
        }
        if let Some(tlskey) = &self.tlskey {
            command.args(["--tlskey", tlskey]);
        }
        if self.tlsverify {
            command.arg("--tlsverify");
        }

        command
    }

    pub fn docker_compose(&self) -> process::Command {
        let mut command = self.docker();
        command.arg("compose");

        for file in &self.file {
            command.args(["--file", file]);
        }
        if let Some(project_directory) = &self.project_directory {
            command.args(["--project-directory", project_directory]);
        }
        if let Some(project_name) = &self.project_name {
            command.args(["--project-name", project_name]);
        }

        command
    }
}
