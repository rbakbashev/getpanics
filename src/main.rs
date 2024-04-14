#![allow(
    clippy::missing_const_for_fn,
    clippy::option_if_let_else,
    clippy::single_match_else,
    clippy::uninlined_format_args
)]

mod args;
mod db;
mod utils;

fn main() {
    let args = args::parse();
    let (targets, db) = db::construct(&args.directory);
    let target = args::choose_target(&args, &targets);
}
