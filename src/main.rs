mod command;
mod deploy;
mod docker;
mod docker_cli_plugin_metadata;
mod log;
mod provision;
mod run_with_ssh_config;

use clap::Parser;
use clap::ValueEnum;
use std::path;

fn main() -> anyhow::Result<()> {
    let Cli {
        docker_arguments,
        subcommand,
    } = Cli::parse();

    log::set_level(match docker_arguments.log_level {
        _ if docker_arguments.debug => log::Level::Debug,
        None => log::Level::Info,
        Some(LogLevel::Debug) => log::Level::Debug,
        Some(LogLevel::Error) => log::Level::Error,
        Some(LogLevel::Fatal) => log::Level::Error,
        Some(LogLevel::Info) => log::Level::Info,
        Some(LogLevel::Warn) => log::Level::Warn,
    })?;

    match subcommand {
        Subcommand::Deploy {
            compose_arguments,
            compose_engine,
            compose_up_arguments:
                ComposeUpArgumentsForDeploy {
                    build,
                    detach,
                    force_recreate,
                    no_build,
                    no_start,
                    pull,
                    quiet_pull,
                    remove_orphans,
                    renew_anon_volumes,
                    timeout,
                    wait_timeout,
                    wait,
                },
            container_engine,
            service_names,
        } => {
            if detach {
                log::warn!("Detached mode is always on, no need to set it.");
            }
            let dry_run = compose_arguments.dry_run;

            deploy::go(deploy::In {
                build,
                docker_cli: docker_cli(
                    docker_arguments,
                    compose_arguments,
                    container_engine,
                    compose_engine,
                ),
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
            })
        }

        Subcommand::DockerCliPluginMetadata => {
            let metadata = docker_cli_plugin_metadata::go()?;
            println!("{metadata}");
            Ok(())
        }

        Subcommand::Provision {
            force,
            host,
            ssh_config,
        } => provision::go(provision::In {
            force,
            host,
            ssh_config,
        }),

        Subcommand::RunWithSshConfig {
            command,
            ssh_config,
        } => run_with_ssh_config::go(run_with_ssh_config::In {
            command,
            ssh_config,
        }),
    }
}

// Order of fields matters for generated help.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(flatten)]
    docker_arguments: DockerArguments,

    #[command(subcommand)]
    subcommand: Subcommand,
}

// Top-level Docker arguments.
//
// Source: https://docs.docker.com/reference/cli/docker/
// TODO: Update arguments based on above source.
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
    #[arg(long, short = 'D', visible_alias = "verbose")]
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

#[allow(clippy::large_enum_variant)]
#[derive(clap::Subcommand)]
enum Subcommand {
    /// Create or update services
    ///
    /// Builds, (re)creates, and starts containers for a service.
    ///
    /// Unless they are already running, this command also starts any linked
    /// services.
    ///
    /// The containers are always started in the background and left running
    /// (detached mode).
    ///
    /// If there are existing containers for a service, and the service's
    /// configuration was changed after the container's creation, then the
    /// changes are picked up by recreating the containers (preserving mounted
    /// volumes). Whether the old containers are stopped before or after
    /// the new containers are started is controlled via
    /// `services.*.deploy.update_config.order` in a Compose file.
    ///
    /// To force recreating all containers, use the `--force-recreate` flag.
    Deploy {
        #[command(flatten)]
        compose_arguments: ComposeArguments,

        #[command(flatten)]
        compose_up_arguments: ComposeUpArgumentsForDeploy,

        /// Compose engine to use; Podman Compose is not supported due to
        /// missing features
        #[arg(default_value_t = ComposeEngine::DockerComposeV2, long, value_enum)]
        compose_engine: ComposeEngine,

        /// Container engine to use
        #[arg(default_value_t = ContainerEngine::Docker, long, value_enum)]
        container_engine: ContainerEngine,

        /// Services to consider
        service_names: Vec<String>,
    },

    #[command(hide = true)]
    DockerCliPluginMetadata,

    /// Provisions host with container engine
    Provision {
        /// Go ahead without prompting user to confirm
        #[arg(long)]
        force: bool,

        /// Path to SSH config file
        #[arg(long, short = 'F')]
        ssh_config: Option<String>,

        /// Reference like "localhost", "[ssh://]<host>", "vagrant://[<vm>]"
        host: String,
    },

    /// Runs command with wrapped `ssh` in `$PATH` that uses given SSH config
    ///
    /// This may be useful for an SSH connection to a Docker host with a custom
    /// SSH config file. The Docker CLI supports the form `ssh://…` to specify
    /// an SSH connection with username, hostname, port, etc. However, a custom
    /// SSH config file other than `~/.ssh/config` cannot be provided.
    RunWithSshConfig {
        /// Path to SSH config file
        ssh_config: path::PathBuf,

        /// Program with arguments to run
        #[arg(required = true)]
        command: Vec<String>,
    },
}

// Top-level Compose arguments.
//
// Source: https://docs.docker.com/reference/cli/docker/compose/
// TODO: Update arguments based on above source.
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

// `docker compose up` arguments, where applicable.
//
// Source: https://docs.docker.com/reference/cli/docker/compose/up/
// TODO: Document arguments that are not applicable.
// TODO: Update arguments based on above source.
#[derive(clap::Args)]
struct ComposeUpArgumentsForDeploy {
    /// Build images before starting containers
    #[arg(long)]
    build: bool,

    /// This has no effect as detached mode is always on; for migration only
    #[arg(long, short = 'd')]
    detach: bool,

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
}

#[derive(Clone, ValueEnum)]
enum Pull {
    Always,
    Missing,
    Never,
}

#[derive(Clone, ValueEnum)]
enum ComposeEngine {
    #[clap(name = "docker-compose")]
    DockerComposeV1,
    #[clap(name = "docker compose")]
    DockerComposeV2,
}

#[derive(Clone, ValueEnum)]
enum ContainerEngine {
    Docker,
    Podman,
}

fn docker_cli(
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
    }: DockerArguments,
    ComposeArguments {
        ansi,
        compatibility,
        dry_run: _,
        env_file,
        file,
        parallel,
        profile,
        progress,
        project_directory,
        project_name,
    }: ComposeArguments,
    container_engine: ContainerEngine,
    compose_engine: ComposeEngine,
) -> docker::Cli {
    docker::Cli::new(
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
        canonical_argument(compose_engine)
            .split_whitespace()
            .map(|part| part.into())
            .collect(),
    )
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

    #[test]
    fn readme_top_level_heading_includes_description() {
        let description = env!("CARGO_PKG_DESCRIPTION");
        let top_level_heading = format!("# Wheelsticks: {description}\n\n");

        assert!(get_readme().starts_with(&top_level_heading));
    }

    fn get_readme() -> &'static str {
        include_str!("../README.md")
    }

    #[test_case::test_case(&[]; "")]
    #[test_case::test_case(&["deploy"]; "deploy")]
    #[test_case::test_case(&["provision"]; "provision")]
    #[test_case::test_case(&["run-with-ssh-config"]; "run-with-ssh-config")]
    fn readme_includes_subcommand_help(subcommands: &[&str]) {
        let help_command = [&[env!("CARGO_BIN_NAME")], subcommands, &["-h"]]
            .concat()
            .join(" ");
        let mut root = Cli::command().term_width(80);
        root.build();
        let leaf = subcommands.iter().fold(&mut root, |node, subcommand| {
            node.find_subcommand_mut(subcommand).expect(subcommand)
        });
        let help_message = leaf.render_help();
        let help_section = format!("\n\n### `{help_command}`\n\n```\n{help_message}```\n");

        assert!(get_readme().contains(&help_section), "{help_section}")
    }
}
