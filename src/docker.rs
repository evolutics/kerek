use super::log;
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
    pub log_level: Option<log::Level>,
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
            .args(context.iter().flat_map(|context| ["--context", context]))
            .args(debug.then_some("--debug").iter())
            .args(host.iter().flat_map(|host| ["--host", host]))
            .args(log_level.iter().flat_map(|log_level| {
                [
                    "--log-level",
                    match log_level {
                        log::Level::Debug => "debug",
                        log::Level::Info => "info",
                        log::Level::Warn => "warn",
                        log::Level::Error => "error",
                        log::Level::Fatal => "fatal",
                    },
                ]
            }))
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

    pub fn default_daemon(&self) -> Self {
        Self {
            arguments: Arguments {
                context: None,
                host: None,
                ..self.arguments
            },
            container_engine: self.container_engine,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn handles_maximum() -> anyhow::Result<()> {
        let command = Cli::new(
            "container-engine",
            Arguments {
                config: Some("config"),
                context: Some("context"),
                debug: true,
                host: Some("host"),
                log_level: Some(log::Level::Warn),
                tls: true,
                tlscacert: Some("tlscacert"),
                tlscert: Some("tlscert"),
                tlskey: Some("tlskey"),
                tlsverify: true,
            },
        )
        .command();

        assert_eq!(command.get_program(), "container-engine");
        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            [
                "--config",
                "config",
                "--context",
                "context",
                "--debug",
                "--host",
                "host",
                "--log-level",
                "warn",
                "--tls",
                "--tlscacert",
                "tlscacert",
                "--tlscert",
                "tlscert",
                "--tlskey",
                "tlskey",
                "--tlsverify",
            ],
        );
        Ok(())
    }

    #[test]
    fn handles_minimum() -> anyhow::Result<()> {
        let command = Cli::new(
            "container-engine",
            Arguments {
                config: None,
                context: None,
                debug: false,
                host: None,
                log_level: None,
                tls: false,
                tlscacert: None,
                tlscert: None,
                tlskey: None,
                tlsverify: false,
            },
        )
        .command();

        assert_eq!(command.get_program(), "container-engine");
        assert_eq!(command.get_args().next(), None);
        Ok(())
    }

    #[test]
    fn handles_default_daemon() -> anyhow::Result<()> {
        let command = Cli::new(
            "container-engine",
            Arguments {
                config: None,
                context: Some("context"),
                debug: true,
                host: Some("host"),
                log_level: None,
                tls: false,
                tlscacert: None,
                tlscert: None,
                tlskey: None,
                tlsverify: false,
            },
        )
        .default_daemon()
        .command();

        assert_eq!(command.get_program(), "container-engine");
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["--debug"]);
        Ok(())
    }
}
