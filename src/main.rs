mod command;
mod deploy;
mod docker;
mod docker_cli_plugin_metadata;
mod docker_compose;
mod log;
mod provision;
mod ssh;
mod transfer_images;
mod tunnel_ssh;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let Cli {
        docker_arguments,
        dry_run,
        subcommand,
    } = Cli::parse();

    log::set_level(if docker_arguments.debug {
        log::Level::Debug
    } else {
        docker_arguments.log_level.unwrap_or(log::Level::Info)
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
            ssh_arguments,
        } => provision::go(provision::In {
            dry_run,
            force,
            has_ssh_config_override: ssh_arguments.ssh_config.is_some(),
            host,
            ssh_cli: ssh_cli(&docker_arguments, &ssh_arguments),
        }),

        Subcommand::TransferImages {
            container_engine_arguments: ContainerEngineArguments { container_engine },
            images,
        } => transfer_images::go(transfer_images::In {
            docker_cli: docker::Cli::new(&container_engine, (&docker_arguments).into()),
            dry_run,
            images,
        }),

        Subcommand::TunnelSsh {
            local_socket,
            remote_socket,
            ssh_arguments,
            ssh_host,
        } => tunnel_ssh::go(tunnel_ssh::In {
            dry_run,
            local_socket,
            remote_socket,
            ssh_cli: ssh_cli(&docker_arguments, &ssh_arguments),
            ssh_host,
        }),
    }
}

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
    #[arg(long, short = 'l', value_enum)]
    log_level: Option<log::Level>,

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

    /// Provisions host with container engine, making system-wide changes
    ///
    /// This targets a host via SSH, unless host "localhost" and no SSH config
    /// file are passed as arguments, in which case the current machine is
    /// targeted.
    Provision {
        /// Go ahead without prompting user to confirm
        #[arg(long)]
        force: bool,

        #[command(flatten)]
        ssh_arguments: SshArguments,

        /// Reference like "localhost" or "[ssh://][<user>@]<hostname>[:<port>]"
        host: String,
    },

    /// Copies images from default to specified Docker host
    ///
    /// Examples:
    ///
    ///     kerek --host ssh://192.0.2.1 transfer-images my-img
    ///     DOCKER_HOST=ssh://from kerek --host ssh://to transfer-images my-img
    ///     DOCKER_CONTEXT=from kerek --context to transfer-images my-img
    ///     docker compose config --images | kerek --host … transfer-images -
    TransferImages {
        #[command(flatten)]
        container_engine_arguments: ContainerEngineArguments,

        /// Images to copy; use "-" to pass image names as stdin lines
        images: Vec<String>,
    },

    /// Forwards local Unix domain socket to remote Docker host over SSH
    ///
    /// This runs an SSH tunnel in the background. Meanwhile, you can connect to
    /// the remote Docker host using `DOCKER_HOST=unix:///path/to/temp.sock`
    /// locally. Note that a custom SSH config file can be specified, unlike
    /// with vanilla Docker.
    ///
    /// Example:
    ///
    ///     kerek tunnel-ssh --local-socket temp.sock my-ssh-host
    ///     DOCKER_HOST="unix://${PWD}/temp.sock" podman ps
    ///     kill "$(lsof -t "${PWD}/temp.sock")"
    TunnelSsh {
        /// Path to Unix domain socket on localhost to be forwarded
        #[arg(default_value = "kerek.sock", long)]
        local_socket: String,

        /// Path to Unix domain socket of Docker host on remote
        #[arg(long)]
        remote_socket: Option<String>,

        #[command(flatten)]
        ssh_arguments: SshArguments,

        /// Reference like "[ssh://][<user>@]<hostname>[:<port>]"
        ssh_host: String,
    },
}

#[derive(clap::Args)]
struct ContainerEngineArguments {
    /// Container engine program to use
    #[arg(default_value = "docker", env, long, value_enum)]
    container_engine: String,
}

#[derive(clap::Args)]
struct SshArguments {
    /// Path to SSH config file
    #[arg(long, short = 'F')]
    ssh_config: Option<String>,
}

// Top-level Docker Compose arguments.
//
// Source:
// https://github.com/docker/compose/blob/main/docs/reference/compose.md#options
//
// Option `--dry-run` is available on shared level.
#[derive(clap::Args)]
struct DockerComposeArguments {
    /// Include all resources, even those not used by services
    #[arg(long)]
    all_resources: bool,

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
    #[arg(long, value_parser = ["auto", "tty", "plain", "json", "quiet"])]
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
            log_level: *log_level,
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
            all_resources,
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
            all_resources: *all_resources,
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

fn ssh_cli<'a>(
    docker_arguments: &'a DockerArguments,
    SshArguments { ssh_config }: &'a SshArguments,
) -> ssh::Cli<'a> {
    ssh::Cli::new(ssh::Arguments {
        config: ssh_config.as_deref(),
        debug: docker_arguments.debug,
        log_level: docker_arguments.log_level,
    })
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
        let top_level_heading = format!("# Kerek: {description}\n\n");

        assert!(get_readme().starts_with(&top_level_heading));
    }

    fn get_readme() -> &'static str {
        include_str!("../README.md")
    }

    #[test_case::test_case(&[]; "")]
    #[test_case::test_case(&["deploy"]; "deploy")]
    #[test_case::test_case(&["provision"]; "provision")]
    #[test_case::test_case(&["transfer-images"]; "transfer-images")]
    #[test_case::test_case(&["tunnel-ssh"]; "tunnel-ssh")]
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
