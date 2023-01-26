mod library;
mod subcommand;

use clap::Parser;
use std::path;
use subcommand::clean;
use subcommand::dry_run;
use subcommand::provision;
use subcommand::run;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    match arguments.subcommand {
        Subcommand::Clean => clean::go(arguments.configuration),
        Subcommand::DryRun => dry_run::go(arguments.configuration),
        Subcommand::Provision => provision::go(arguments.configuration),
        Subcommand::Run => run::go(arguments.configuration),
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Path to configuration file.
    #[arg(default_value = "kerek.json", long)]
    configuration: path::PathBuf,

    #[command(subcommand)]
    subcommand: Subcommand,
}

#[derive(clap::Subcommand)]
enum Subcommand {
    /// Removes internal resources such as the cache folder.
    Clean,
    /// Builds, tests, deploys once to staging only.
    DryRun,
    /// Applies provision script to production.
    Provision,
    /// Builds, tests, deploys in a loop.
    Run,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::CommandFactory;

    #[test]
    fn cli_verifies() {
        Arguments::command().debug_assert()
    }
}
