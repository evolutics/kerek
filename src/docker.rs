use std::process;

pub struct Cli<'a> {
    arguments: Arguments<'a>,
    container_engine: &'a str,
}

pub struct Arguments<'a> {
    pub config: Option<&'a str>,
    pub context: Option<&'a str>,
    pub debug: bool,
    pub host: Option<&'a str>,
    pub log_level: Option<&'a str>,
    pub tls: bool,
    pub tlscacert: Option<&'a str>,
    pub tlscert: Option<&'a str>,
    pub tlskey: Option<&'a str>,
    pub tlsverify: bool,
}

impl<'a> Cli<'a> {
    pub fn new(container_engine: &'a str, arguments: Arguments<'a>) -> Self {
        Self {
            arguments,
            container_engine,
        }
    }

    pub fn command(&self) -> process::Command {
        self.base(false)
    }

    fn base(&self, default_daemon: bool) -> process::Command {
        let mut command = process::Command::new(self.container_engine);

        let Arguments {
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
        } = &self.arguments;

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

    pub fn command_default_daemon(&self) -> process::Command {
        self.base(true)
    }
}
