mod clean;
mod inspect;
mod run;

use clap::Parser;

fn main() {
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
