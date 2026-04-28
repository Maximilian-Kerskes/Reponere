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
    Install {
        #[arg(required = true)]
        packages: Vec<String>,
    },
    Uninstall {
        #[arg(required = true)]
        packages: Vec<String>,
    },
    Update {
        #[arg(required = false)]
        packages: Vec<String>,
    },
    Sync,
    List {
        #[arg(required = false)]
        packages: Vec<String>,

        #[arg(long)]
        available: bool,
    },
    Show {
        #[arg(required = true)]
        package: String,
    },
}
