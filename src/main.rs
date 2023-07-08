mod library;
mod subcommand;

use clap::Parser;
use std::path;
use subcommand::build;
use subcommand::deploy;
use subcommand::provision;

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.subcommand {
        Subcommand::Build {
            compose:
                Compose {
                    compose_file,
                    project_folder,
                    project_name,
                },
        } => build::go(build::In {
            compose_file,
            project_folder,
            project_name,
        }),

        Subcommand::Deploy {
            compose:
                Compose {
                    compose_file,
                    project_folder,
                    project_name,
                },
            docker_host,
        } => deploy::go(deploy::In {
            compose_file,
            docker_host: docker_host.host,
            project_folder,
            project_name,
        }),

        Subcommand::Provision {
            deploy_user,
            docker_host,
            upgrade_packages,
        } => provision::go(provision::In {
            deploy_user,
            docker_host: docker_host.host,
            upgrade_packages,
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
    Build {
        #[command(flatten)]
        compose: Compose,
    },
    Deploy {
        #[command(flatten)]
        compose: Compose,

        #[command(flatten)]
        docker_host: DockerHost,
    },
    Provision {
        #[arg(default_value = "wheelsticks", long)]
        deploy_user: String,

        #[command(flatten)]
        docker_host: DockerHost,

        #[arg(long)]
        upgrade_packages: bool,
    },
}

#[derive(clap::Args)]
struct Compose {
    #[arg(default_value = "compose.yaml", long, short = 'f')]
    compose_file: path::PathBuf,
    #[arg(long = "project-directory")]
    project_folder: Option<path::PathBuf>,
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
