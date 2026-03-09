mod build;
mod cli;
mod handlers;
mod util;

use crate::{cli::commands, util::context::Context};

fn main() {
    let mut ctx = Context::new().unwrap_or_else(|e| {
        eprintln!("==> something went wrong: {e}");
        std::process::exit(1);
    });

    commands::run(&mut ctx);
}
