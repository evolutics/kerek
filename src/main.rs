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
                    project_name,
                },
        } => build::go(build::In {
            compose_file,
            project_name,
        }),

        Subcommand::Deploy {
            compose:
                Compose {
                    compose_file,
                    project_name,
                },
            ssh: Ssh {
                ssh_configuration,
                ssh_user,
            },
            ssh_host,
        } => deploy::go(deploy::In {
            compose_file,
            project_name,
            ssh_configuration,
            ssh_host,
            ssh_user,
        }),

        Subcommand::Provision {
            deploy_user,
            ssh: Ssh {
                ssh_configuration,
                ssh_user,
            },
            ssh_host,
            upgrade_packages,
        } => provision::go(provision::In {
            deploy_user,
            ssh_configuration,
            ssh_host,
            ssh_user,
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
        ssh: Ssh,

        ssh_host: Option<String>,
    },
    Provision {
        #[arg(default_value = "wheelsticks", long)]
        deploy_user: String,

        #[command(flatten)]
        ssh: Ssh,

        #[arg(long)]
        upgrade_packages: bool,

        ssh_host: String,
    },
}

#[derive(clap::Args)]
struct Compose {
    #[arg(default_value = "compose.yaml", long, short = 'f')]
    compose_file: path::PathBuf,
    #[arg(long, short = 'p')]
    project_name: Option<String>,
}

#[derive(clap::Args)]
struct Ssh {
    #[arg(long, short = 'F')]
    ssh_configuration: Option<path::PathBuf>,

    #[arg(long)]
    ssh_user: Option<String>,
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
