use clap::{AppSettings, Clap};

mod build;

use build::BuildCmd;

#[derive(Clap)]
#[clap(version = "0.0.1", author = "Mathias Pius <contact@pius.io>")]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(subcommand)]
    subcmd: SubCommand,
}

#[derive(Clap)]
enum SubCommand {
    Build(BuildCmd),
}

fn main() {
    println!("Hello, world!");
}
