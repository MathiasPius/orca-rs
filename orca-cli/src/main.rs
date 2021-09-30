use clap::{AppSettings, Clap};

mod build;
mod cache;
mod identifier;

use build::BuildCmd;

#[derive(Clap)]
#[clap(version = "0.0.1", author = "Mathias Pius <contact@pius.io>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(long, short, default_value = ".orca/cache")]
    cache_directory: String,
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Build(BuildCmd),
}

fn main() {
    let opts = Opts::parse();

    match &opts.subcmd {
        SubCommand::Build(build) => build.execute(&opts),
    }
}
