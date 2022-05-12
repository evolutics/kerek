mod action;
mod library;

use action::clean;
use action::diagnose;
use action::run;
use clap::Parser;
use std::path;

fn main() -> anyhow::Result<()> {
    let arguments = Arguments::parse();

    match arguments.action {
        Action::Clean => clean::go(arguments.configuration),
        Action::Diagnose => diagnose::go(),
        Action::Run => run::go(arguments.configuration),
    }
}

#[derive(Parser)]
#[clap(version)]
struct Arguments {
    #[clap(default_value = "kerek.json", long, parse(from_os_str))]
    configuration: path::PathBuf,

    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    Clean,
    Diagnose,
    Run,
}
