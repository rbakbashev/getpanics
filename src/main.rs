#![allow(
    clippy::single_match_else,
    clippy::option_if_let_else,
    clippy::missing_const_for_fn
)]

mod args;
mod db;
mod utils;

fn main() {
    let args = args::parse();
    let _db = db::construct(args.directory);
}
