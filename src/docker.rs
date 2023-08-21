use std::process;

pub struct Cli {
    file: Vec<String>,
    host: Option<String>,
    project_directory: Option<String>,
    project_name: Option<String>,
}

pub struct In {
    pub file: Vec<String>,
    pub host: Option<String>,
    pub project_directory: Option<String>,
    pub project_name: Option<String>,
}

impl Cli {
    pub fn new(
        In {
            file,
            host,
            project_directory,
            project_name,
        }: In,
    ) -> Self {
        Cli {
            file,
            host,
            project_directory,
            project_name,
        }
    }

    pub fn docker(&self) -> process::Command {
        let mut command = process::Command::new("docker");
        if let Some(host) = &self.host {
            command.args(["--host", host]);
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
