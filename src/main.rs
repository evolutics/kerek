mod command;
mod deploy;
mod docker;

use clap::Parser;

fn main() -> anyhow::Result<()> {
    let Cli {
        docker_arguments: DockerArguments { host },
        subcommand,
    } = Cli::parse();

    match subcommand {
        Subcommand::Deploy {
            compose_arguments:
                ComposeArguments {
                    file,
                    project_directory,
                    project_name,
                },
        } => deploy::go(deploy::In {
            docker_cli: docker::Cli::new(docker::In { host }),
            file,
            project_directory,
            project_name,
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
    #[arg(long, short = 'H')]
    host: Option<String>,
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
    // TODO: Support more Docker standard arguments, e.g. `--context`.
    // TODO: Support use as plugin (https://github.com/docker/cli/issues/1534).
    Deploy {
        #[command(flatten)]
        compose_arguments: ComposeArguments,
    },
}

#[derive(clap::Args)]
struct ComposeArguments {
    #[arg(long, short = 'f')]
    file: Vec<String>,
    #[arg(long)]
    project_directory: Option<String>,
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
