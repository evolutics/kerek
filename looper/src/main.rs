mod action;
mod library;

use action::clean;
use action::dry_run;
use action::provision;
use action::run;
use clap::Parser;
use std::path;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    match arguments.action {
        Action::Clean => clean::go(arguments.configuration),
        Action::DryRun => dry_run::go(arguments.configuration),
        Action::Provision => provision::go(arguments.configuration),
        Action::Run => run::go(arguments.configuration),
    }
}

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Arguments {
    /// Path to configuration file.
    #[arg(default_value = "kerek.json", long)]
    configuration: path::PathBuf,

    #[command(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    /// Tears down internal resources such as the cache folder.
    Clean,
    /// Builds, tests, deploys to staging in a loop.
    DryRun,
    /// Sets up the production environment for the first time.
    Provision,
    /// Builds, tests, deploys to staging, then to production, in a loop.
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
