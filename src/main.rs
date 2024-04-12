#![allow(clippy::single_match_else, clippy::option_if_let_else)]

mod args;
mod utils;

fn main() {
    let args = args::parse();

    println!("args.directory={:?}", args.directory);
}
