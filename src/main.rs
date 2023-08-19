// TODO: Flatten module tree.
mod library;
mod subcommand;

use clap::Parser;
use subcommand::deploy;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        Subcommand::Deploy {
            compose:
                Compose {
                    compose_file,
                    project_folder,
                    project_name,
                },
            docker_host: DockerHost { host },
        } => deploy::go(deploy::In {
            compose_file,
            docker_host: host,
            project_folder,
            project_name,
        }),
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    subcommand: Subcommand,
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
        compose: Compose,

        #[command(flatten)]
        docker_host: DockerHost,
    },
}

#[derive(clap::Args)]
struct Compose {
    #[arg(default_value = "compose.yaml", long, short = 'f')] // TODO: Remove default.
    compose_file: String,
    #[arg(long = "project-directory")]
    project_folder: Option<String>,
    #[arg(long, short = 'p')]
    project_name: Option<String>,
}

#[derive(clap::Args)]
struct DockerHost {
    #[arg(long, short = 'H')]
    host: Option<String>,
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
