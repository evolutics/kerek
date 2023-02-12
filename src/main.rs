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
        Subcommand::Build => build::go(cli.compose_file),

        Subcommand::Deploy {
            ssh_configuration,
            ssh_host,
        } => deploy::go(deploy::In {
            configuration: cli.compose_file,
            ssh_configuration,
            ssh_host,
        }),

        Subcommand::Provision {
            ssh_configuration,
            ssh_host,
        } => provision::go(provision::In {
            configuration: cli.compose_file,
            ssh_configuration,
            ssh_host,
        }),
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[arg(default_value = "compose.yaml", global = true, long, short = 'f')]
    compose_file: path::PathBuf,

    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    Build,
    Deploy {
        #[arg(long, short = 'F')]
        ssh_configuration: Option<path::PathBuf>,

        ssh_host: Option<String>,
    },
    Provision {
        #[arg(long, short = 'F')]
        ssh_configuration: Option<path::PathBuf>,

        ssh_host: String,
    },
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
