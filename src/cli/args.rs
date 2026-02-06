use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(
    name = "reponere",
    version,
    about = "A Rust package manager",
    long_about = None
)]
pub struct Arg {
    #[command(subcommand)]
    pub sub: SubArgs,
}

#[derive(Subcommand, Debug)]
pub enum SubArgs {
    Install { packages: Vec<String> },
    Uninstall { packages: Vec<String> },
}
