mod action;

use action::clean;
use action::inspect;
use action::run;
use clap::Parser;

fn main() -> Result<(), String> {
    let arguments = Arguments::parse();

    match arguments.action {
        Action::Clean => clean::go(),
        Action::Inspect => inspect::go(),
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
    Inspect,
    Run,
}
