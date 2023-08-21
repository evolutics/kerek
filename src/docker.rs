use std::process;

pub struct Cli {
    host: Option<String>,
}

pub struct In {
    pub host: Option<String>,
}

impl Cli {
    pub fn new(In { host }: In) -> Self {
        Cli { host }
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
        command
    }
}
