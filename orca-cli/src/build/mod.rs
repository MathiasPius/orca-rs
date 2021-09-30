mod deps;
mod spec;

use std::path::PathBuf;

use clap::Clap;
use indoc::indoc;

use crate::Opts;

#[derive(Clap)]
pub struct BuildCmd {
    #[clap(long, about = "Path(s) to one or more BuildSpec json files.", long_about = indoc!{"
        A BuildSpec Json file contains one or more build specifications.
        Multiple BuildSpec files can be provided, and dependencies will be resolved automatically.
    "})]
    pub spec: Vec<String>,
}

impl BuildCmd {
    pub(crate) fn execute(&self, opts: &Opts) {}
}
