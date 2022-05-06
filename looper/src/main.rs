mod action;
mod library;

use action::clear;
use action::diagnose;
use action::run;
use clap::Parser;

fn main() -> Result<(), String> {
    let arguments = Arguments::parse();

    match arguments.action {
        Action::Clear => clear::go(),
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
    Clear,
    Diagnose,
    Run,
}
