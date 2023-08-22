mod command;
mod deploy;
mod docker;

use clap::Parser;
use clap::ValueEnum;

fn main() -> anyhow::Result<()> {
    let Cli {
        docker_arguments:
            DockerArguments {
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
            },
        subcommand,
    } = Cli::parse();
    let docker_arguments = docker::DockerArguments {
        config,
        context,
        debug,
        host,
        log_level: log_level
            .and_then(|level| level.to_possible_value())
            .map(|level| level.get_name().into()),
        tls,
        tlscacert,
        tlscert,
        tlskey,
        tlsverify,
    };

    match subcommand {
        Subcommand::Deploy {
            compose_arguments:
                ComposeArguments {
                    file,
                    project_directory,
                    project_name,
                },
        } => deploy::go(deploy::In {
            docker_cli: docker::Cli::new(
                docker_arguments,
                docker::ComposeArguments {
                    file,
                    project_directory,
                    project_name,
                },
            ),
        }),
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    docker_arguments: DockerArguments,

    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Args)]
struct DockerArguments {
    /// Location of client config files
    #[arg(long)]
    config: Option<String>,

    /// Name of the context to use to connect to the daemon (overrides
    /// DOCKER_HOST env var and default context set with "docker context use")
    #[arg(long, short = 'c')]
    context: Option<String>,

    /// Enable debug mode
    #[arg(long, short = 'D')]
    debug: bool,

    /// Daemon socket to connect to
    #[arg(long, short = 'H')]
    host: Option<String>,

    /// Set the logging level
    #[arg(long, short = 'l', value_enum)]
    log_level: Option<LogLevel>,

    /// Use TLS; implied by --tlsverify
    #[arg(long)]
    tls: bool,

    /// Trust certs signed only by this CA
    #[arg(long)]
    tlscacert: Option<String>,

    /// Path to TLS certificate file
    #[arg(long)]
    tlscert: Option<String>,

    /// Path to TLS key file
    #[arg(long)]
    tlskey: Option<String>,

    /// Use TLS and verify the remote
    #[arg(long)]
    tlsverify: bool,
}

#[derive(Clone, ValueEnum)]
enum LogLevel {
    Debug,
    Info,
    Warn,
    Error,
    Fatal,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    // TODO: Support collecting garbage with `system prune --all --force --volumes`.
    // TODO: Support dry run.
    // TODO: Support forced update.
    // TODO: Support limiting to given service names.
    // TODO: Support maintaining systemd units.
    // TODO: Support more Docker Compose `up` arguments, e.g. `--build`.
    // TODO: Support more Docker Compose standard arguments, e.g. `--env-file`.
    // TODO: Support use as plugin (https://github.com/docker/cli/issues/1534).
    Deploy {
        #[command(flatten)]
        compose_arguments: ComposeArguments,
    },
}

#[derive(clap::Args)]
struct ComposeArguments {
    /// Compose configuration files
    #[arg(long, short = 'f')]
    file: Vec<String>,

    /// Specify an alternate working directory (default: the path of the, first
    /// specified, Compose file)
    #[arg(long)]
    project_directory: Option<String>,

    /// Project name
    #[arg(long, short = 'p')]
    project_name: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_verifies() {
        Cli::command().debug_assert()
    }
}
