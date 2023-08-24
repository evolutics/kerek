mod command;
mod deploy;
mod docker;

use clap::Parser;
use clap::ValueEnum;

fn main() -> anyhow::Result<()> {
    let Cli {
        compose_arguments:
            ComposeArguments {
                ansi,
                compatibility,
                dry_run,
                env_file,
                file,
                parallel,
                profile,
                progress,
                project_directory,
                project_name,
            },
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

    let docker_cli = docker::Cli::new(
        docker::DockerArguments {
            config,
            context,
            debug,
            host,
            log_level: log_level.and_then(canonical_argument),
            tls,
            tlscacert,
            tlscert,
            tlskey,
            tlsverify,
        },
        docker::ComposeArguments {
            ansi: ansi.and_then(canonical_argument),
            compatibility,
            env_file,
            file,
            parallel,
            profile,
            progress: progress.and_then(canonical_argument),
            project_directory,
            project_name,
        },
    );

    match subcommand {
        Subcommand::Deploy => deploy::go(deploy::In {
            docker_cli,
            dry_run,
        }),
    }
}

// Order of fields matters for generated help.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    docker_arguments: DockerArguments,

    #[command(flatten)]
    compose_arguments: ComposeArguments,

    #[command(subcommand)]
    subcommand: Subcommand,
}

/// Top-level Docker arguments.
///
/// Source: https://docs.docker.com/engine/reference/commandline/cli/
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

/// Top-level Compose arguments.
///
/// Source: https://docs.docker.com/engine/reference/commandline/compose/
#[derive(clap::Args)]
struct ComposeArguments {
    /// Control when to print ANSI control characters
    #[arg(long, value_enum)]
    ansi: Option<Ansi>,

    /// Run compose in backward compatibility mode
    #[arg(long)]
    compatibility: bool,

    /// Execute command in dry run mode
    #[arg(long)]
    dry_run: bool,

    /// Specify an alternate environment file
    #[arg(long)]
    env_file: Vec<String>,

    /// Compose configuration files
    #[arg(long, short = 'f')]
    file: Vec<String>,

    /// Control max parallelism, -1 for unlimited
    #[arg(long)]
    parallel: Option<i16>,

    /// Specify a profile to enable
    #[arg(long)]
    profile: Vec<String>,

    /// Set type of progress output
    #[arg(long, value_enum)]
    progress: Option<Progress>,

    /// Specify an alternate working directory (default: the path of the, first
    /// specified, Compose file)
    #[arg(long)]
    project_directory: Option<String>,

    /// Project name
    #[arg(long, short = 'p')]
    project_name: Option<String>,
    // Do not support `--verbose` as it just translates to `--debug`.
}

#[derive(Clone, ValueEnum)]
enum Ansi {
    Never,
    Always,
    Auto,
}

#[derive(Clone, ValueEnum)]
enum Progress {
    Auto,
    Tty,
    Plain,
    Quiet,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    // TODO: Support collecting garbage with `system prune --all --force --volumes`.
    // TODO: Support forced update.
    // TODO: Support limiting to given service names.
    // TODO: Support maintaining systemd units.
    // TODO: Support more Docker Compose `up` arguments, e.g. `--build`.
    // TODO: Support use as plugin (https://github.com/docker/cli/issues/1534).
    Deploy,
}

fn canonical_argument<T: ValueEnum>(value: T) -> Option<String> {
    value
        .to_possible_value()
        .map(|value| value.get_name().into())
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
