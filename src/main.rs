mod command;
mod deploy;
mod docker;
mod docker_cli_plugin_metadata;
mod docker_compose;
mod log;
mod provision;
mod run_with_ssh_config;
mod transfer_images;

use clap::Parser;
use std::path;

fn main() -> anyhow::Result<()> {
    let Cli {
        docker_arguments,
        dry_run,
        subcommand,
    } = Cli::parse();

    log::set_level(match docker_arguments.log_level.as_deref() {
        _ if docker_arguments.debug => log::Level::Debug,
        None => log::Level::Info,
        Some(DEBUG) => log::Level::Debug,
        Some(INFO) => log::Level::Info,
        Some(WARN) => log::Level::Warn,
        Some(_) => log::Level::Error,
    })?;

    match subcommand {
        Subcommand::Deploy {
            container_engine_arguments: ContainerEngineArguments { container_engine },
            docker_compose_arguments,
            docker_compose_up_arguments:
                DockerComposeUpArgumentsForDeploy {
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
            service_names,
        } => {
            if detach {
                log::warn!("Detached mode is always on, no need to set it.");
            }

            deploy::go(deploy::In {
                build,
                docker_cli: docker::Cli::new(&container_engine, (&docker_arguments).into()),
                docker_compose_cli: docker_compose::Cli::new(
                    (&docker_arguments).into(),
                    (&docker_compose_arguments).into(),
                ),
                dry_run,
                force_recreate,
                no_build,
                no_start,
                pull,
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
            dry_run,
            force,
            host,
            ssh_config,
        }),

        Subcommand::RunWithSshConfig {
            command,
            ssh_config,
        } => run_with_ssh_config::go(run_with_ssh_config::In {
            command,
            dry_run,
            ssh_config,
        }),

        Subcommand::TransferImages {
            container_engine_arguments: ContainerEngineArguments { container_engine },
            images,
        } => transfer_images::go(transfer_images::In {
            docker_cli: docker::Cli::new(&container_engine, (&docker_arguments).into()),
            dry_run,
            images,
        }),
    }
}

const DEBUG: &str = "debug";
const INFO: &str = "info";
const WARN: &str = "warn";
const ERROR: &str = "error";
const FATAL: &str = "fatal";

// Order of fields matters for generated help.
#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// Do not apply changes, only show what would be done
    #[arg(long)]
    dry_run: bool,

    #[command(flatten)]
    docker_arguments: DockerArguments,

    #[command(subcommand)]
    subcommand: Subcommand,
}

// Top-level Docker arguments. These must be top-level arguments of our CLI for
// its subcommands to be valid Docker plugins even if not always applicable.
//
// Source: https://docs.docker.com/reference/cli/docker/
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
    #[arg(long, short = 'l', value_parser = [DEBUG, INFO, WARN, ERROR, FATAL])]
    log_level: Option<String>,

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
        container_engine_arguments: ContainerEngineArguments,

        #[command(flatten)]
        docker_compose_arguments: DockerComposeArguments,

        #[command(flatten)]
        docker_compose_up_arguments: DockerComposeUpArgumentsForDeploy,

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

    /// Copies images from default to specified Docker host
    ///
    /// Examples:
    ///
    ///     wheelsticks --host ssh://192.0.2.1 transfer-images my-img
    ///     DOCKER_HOST=ssh://from wheelsticks --host ssh://to transfer-images my-img
    ///     DOCKER_CONTEXT=from wheelsticks --context to transfer-images my-img
    ///     docker compose config --images | wheelsticks --host … transfer-images -
    TransferImages {
        #[command(flatten)]
        container_engine_arguments: ContainerEngineArguments,

        /// Images to copy; use "-" to pass image names as stdin lines
        images: Vec<String>,
    },
}

#[derive(clap::Args)]
struct ContainerEngineArguments {
    /// Container engine program to use
    #[arg(default_value = "docker", long, value_enum)]
    container_engine: String,
}

// Top-level Docker Compose arguments.
//
// Source: https://docs.docker.com/reference/cli/docker/compose/
// TODO: Update arguments based on above source.
#[derive(clap::Args)]
struct DockerComposeArguments {
    /// Control when to print ANSI control characters
    #[arg(long, value_parser = ["never", "always", "auto"])]
    ansi: Option<String>,

    /// Run compose in backward compatibility mode
    #[arg(long)]
    compatibility: bool,

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
    #[arg(long, value_parser = ["auto", "tty", "plain", "quiet"])]
    progress: Option<String>,

    /// Specify an alternate working directory (default: the path of the, first
    /// specified, Compose file)
    #[arg(long)]
    project_directory: Option<String>,

    /// Project name
    #[arg(long, short = 'p')]
    project_name: Option<String>,
}

// `docker compose up` arguments, where applicable.
//
// Source: https://docs.docker.com/reference/cli/docker/compose/up/
// TODO: Document arguments that are not applicable.
// TODO: Update arguments based on above source.
#[derive(clap::Args)]
struct DockerComposeUpArgumentsForDeploy {
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
    #[arg(long, value_parser = ["always", "missing", "never"])]
    pull: Option<String>,

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

impl<'a> From<&'a DockerArguments> for docker::Arguments<'a> {
    fn from(
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
        }: &'a DockerArguments,
    ) -> Self {
        docker::Arguments {
            config: config.as_deref(),
            context: context.as_deref(),
            debug: *debug,
            host: host.as_deref(),
            log_level: log_level.as_deref(),
            tls: *tls,
            tlscacert: tlscacert.as_deref(),
            tlscert: tlscert.as_deref(),
            tlskey: tlskey.as_deref(),
            tlsverify: *tlsverify,
        }
    }
}

impl<'a> From<&'a DockerComposeArguments> for docker_compose::Arguments<'a> {
    fn from(
        DockerComposeArguments {
            ansi,
            compatibility,
            env_file,
            file,
            parallel,
            profile,
            progress,
            project_directory,
            project_name,
        }: &'a DockerComposeArguments,
    ) -> Self {
        docker_compose::Arguments {
            ansi: ansi.as_deref(),
            compatibility: *compatibility,
            env_file,
            file,
            parallel: *parallel,
            profile,
            progress: progress.as_deref(),
            project_directory: project_directory.as_deref(),
            project_name: project_name.as_deref(),
        }
    }
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
    #[test_case::test_case(&["transfer-images"]; "transfer-images")]
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
