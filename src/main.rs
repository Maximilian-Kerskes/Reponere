mod build;
mod cli;
mod handlers;
mod util;

use crate::{cli::commands, util::context::Context};

fn main() {
    let mut ctx = Context::new().unwrap();

    commands::run(&mut ctx);
}
