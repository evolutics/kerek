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
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn translates_maximum_arguments() -> anyhow::Result<()> {
        let command = Cli::new(Arguments {
            config: Some("config"),
            debug: true,
            log_level: Some(log::Level::Warn),
        })
        .command();

        assert_eq!(
            command.get_args().collect::<Vec<_>>(),
            ["-F", "config", "-o", "LogLevel=ERROR", "-vvv"],
        );
        Ok(())
    }

    #[test]
    fn translates_minimum_arguments() -> anyhow::Result<()> {
        let command = Cli::new(Arguments {
            config: None,
            debug: false,
            log_level: None,
        })
        .command();

        assert_eq!(command.get_args().next(), None);
        Ok(())
    }
}
