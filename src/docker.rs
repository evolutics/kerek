use std::process;

pub struct Cli {
    docker_host: Option<String>,
}

pub struct In {
    pub docker_host: Option<String>,
}

impl Cli {
    pub fn new(In { docker_host }: In) -> Self {
        Cli { docker_host }
    }

    pub fn docker(&self) -> process::Command {
        let mut command = process::Command::new("docker");
        if let Some(docker_host) = &self.docker_host {
            command.args(["--host", docker_host]);
        }
        command
    }

    pub fn docker_compose(&self) -> process::Command {
        let mut command = self.docker();
        command.arg("compose");
        command
    }
}
