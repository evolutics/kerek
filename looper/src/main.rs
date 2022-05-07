mod action;
mod library;

use action::clean;
use action::diagnose;
use action::run;
use clap::Parser;

fn main() -> Result<(), String> {
    let arguments = Arguments::parse();

    match arguments.action {
        Action::Clean => clean::go(),
        Action::Diagnose => diagnose::go(),
        Action::Run => run::go(),
    }
}

#[derive(Parser)]
#[clap(version)]
struct Arguments {
    #[clap(subcommand)]
    action: Action,
}

#[derive(clap::Subcommand)]
enum Action {
    Clean,
    Diagnose,
    Run,
}
