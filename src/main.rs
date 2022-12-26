mod action;
mod library;

use action::clean;
use action::provision;
use action::run;
use clap::Parser;
use std::path;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    match arguments.action {
        Action::Clean => clean::go(arguments.configuration),
        Action::Provision => provision::go(arguments.configuration),
        Action::Run => run::go(arguments.configuration),
    }
}

#[derive(Parser)]
#[clap(version)]
struct Arguments {
    #[clap(default_value = "kerek.json", long, value_parser)]
    configuration: path::PathBuf,

    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    /// Tears down internal resources such as the cache folder.
    Clean,
    /// Sets up the production environment for the first time.
    Provision,
    /// Builds, tests, deploys in a loop.
    Run,
}
