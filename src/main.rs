mod command;
mod deploy;
mod docker;
mod log;

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
        container_engine,
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

    let is_log_level_debug_or_info = debug
        || match log_level {
            None | Some(LogLevel::Debug) | Some(LogLevel::Info) => true,
            Some(LogLevel::Error) | Some(LogLevel::Fatal) | Some(LogLevel::Warn) => false,
        };
    log::set_level(is_log_level_debug_or_info)?;

    let docker_cli = docker::Cli::new(
        docker::DockerArguments {
            config,
            context,
            debug,
            host,
            log_level: log_level.map(canonical_argument),
            tls,
            tlscacert,
            tlscert,
            tlskey,
            tlsverify,
        },
        docker::ComposeArguments {
            ansi: ansi.map(canonical_argument),
            compatibility,
            env_file,
            file,
            parallel: parallel.map(|parallel| parallel.to_string()),
            profile,
            progress: progress.map(canonical_argument),
            project_directory,
            project_name,
        },
        canonical_argument(container_engine),
    );

    match subcommand {
        Subcommand::Deploy {
            build,
            force_recreate,
            no_build,
            no_start,
            pull,
            quiet_pull,
            remove_orphans,
            renew_anon_volumes,
            service_names,
            timeout,
            wait,
            wait_timeout,
        } => deploy::go(deploy::In {
            build,
            docker_cli,
            dry_run,
            force_recreate,
            no_build,
            no_start,
            pull: pull.map(canonical_argument),
            quiet_pull,
            remove_orphans,
            renew_anon_volumes,
            service_names: service_names.into_iter().collect(),
            timeout: timeout.map(|timeout| timeout.to_string()),
            wait,
            wait_timeout: wait_timeout.map(|wait_timeout| wait_timeout.to_string()),
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

    /// Container engine to use
    #[arg(default_value_t = ContainerEngine::Docker, long, value_enum)]
    container_engine: ContainerEngine,

    #[command(subcommand)]
    subcommand: Subcommand,
}

// Top-level Docker arguments.
//
// Source: https://docs.docker.com/engine/reference/commandline/cli/
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

// Top-level Compose arguments.
//
// Source: https://docs.docker.com/engine/reference/commandline/compose/
//
// Do not support `--verbose` as it just translates to `--debug`.
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

#[derive(Clone, ValueEnum)]
enum ContainerEngine {
    Docker,
    Podman,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    // Source for some arguments:
    // https://docs.docker.com/engine/reference/commandline/compose_up/

    // TODO: Support use as plugin (https://github.com/docker/cli/issues/1534).
    /// Update containers
    Deploy {
        /// Build images before starting containers
        #[arg(long)]
        build: bool,

        /// Recreate containers even if their configuration hasn't changed
        #[arg(long)]
        force_recreate: bool,

        /// Don't build an image, even if it's missing
        #[arg(long)]
        no_build: bool,

        /// Don't start the services after creating them
        #[arg(long)]
        no_start: bool,

        /// Pull image before running
        #[arg(long, value_enum)]
        pull: Option<Pull>,

        /// Pull without printing progress information
        #[arg(long)]
        quiet_pull: bool,

        /// Remove containers for services not defined in the Compose file
        #[arg(long)]
        remove_orphans: bool,

        /// Recreate anonymous volumes instead of retrieving data from the
        /// previous containers
        #[arg(long, short = 'V')]
        renew_anon_volumes: bool,

        /// Use this timeout in seconds for container shutdown when containers
        /// are already running
        #[arg(long, short = 't')]
        timeout: Option<i64>,

        /// Wait for services to be running|healthy
        #[arg(long)]
        wait: bool,

        /// timeout in seconds waiting for application to be running|healthy
        #[arg(long)]
        wait_timeout: Option<i64>,

        service_names: Vec<String>,
    },
}

#[derive(Clone, ValueEnum)]
enum Pull {
    Always,
    Missing,
    Never,
}

fn canonical_argument<T: ValueEnum>(value: T) -> String {
    value
        .to_possible_value()
        .expect("Assertion error: skipped variant unexpected")
        .get_name()
        .into()
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
