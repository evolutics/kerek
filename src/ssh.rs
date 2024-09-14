use super::log;
use std::process;

pub struct Cli<'a> {
    arguments: Arguments<'a>,
}

pub struct Arguments<'a> {
    pub config: Option<&'a str>,
    pub debug: bool,
    pub log_level: Option<log::Level>,
}

impl<'a> Cli<'a> {
    pub fn new(arguments: Arguments<'a>) -> Self {
        Self { arguments }
    }

    pub fn command(&self) -> process::Command {
        let mut command = process::Command::new("ssh");

        let Arguments {
            config,
            debug,
            log_level,
        } = &self.arguments;

        command
            .args(config.iter().flat_map(|config| ["-F", config]))
            .args(log_level.iter().flat_map(|log_level| {
                ["-o".into(), {
                    let level = match log_level {
                        log::Level::Debug => "DEBUG",
                        log::Level::Info => "INFO",
                        log::Level::Warn | log::Level::Error => "ERROR",
                        log::Level::Fatal => "FATAL",
                    };
                    format!("LogLevel={level}")
                }]
            }))
            .args(debug.then_some("-vvv").iter());

        command
    }

    pub fn has_config_override(&self) -> bool {
        self.arguments.config.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_maximum_arguments() -> anyhow::Result<()> {
        let cli = Cli::new(Arguments {
            config: Some("config"),
            debug: true,
            log_level: Some(log::Level::Warn),
        });

        assert_eq!(
            cli.command().get_args().collect::<Vec<_>>(),
            ["-F", "config", "-o", "LogLevel=ERROR", "-vvv"],
        );
        assert!(cli.has_config_override());
        Ok(())
    }

    #[test]
    fn translates_minimum_arguments() -> anyhow::Result<()> {
        let cli = Cli::new(Arguments {
            config: None,
            debug: false,
            log_level: None,
        });

        assert_eq!(cli.command().get_args().next(), None);
        assert!(!cli.has_config_override());
        Ok(())
    }
}
