use clap::Clap;
use indoc::indoc;

#[derive(Clap)]
pub struct BuildCmd {
    #[clap(long, about = "short", long_about = indoc!{"
        looooong
    "})]
    pub path: Option<String>,
}
