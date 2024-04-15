use std::path::PathBuf;
use std::process::exit;

use ra_ap_project_model as ra;

use crate::die;
use crate::utils::{self, MaybeError};

pub struct Args {
    pub directory: PathBuf,
    pub crate_filter: Option<String>,
}

pub fn parse() -> Args {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();

    let mut cursor = refs.as_slice();
    let mut directory = None;
    let mut crate_filter = None;

    loop {
        match cursor {
            ["-h" | "--help", ..] => usage(0),
            ["-v" | "--version", ..] => version(),
            ["-c" | "--crate", crate_filter_arg, rest @ ..] => {
                if let Some(crate_filter_cur) = crate_filter {
                    eprintln!(
                        "multiple crates provided: '{crate_filter_cur}' and '{crate_filter_arg}'"
                    );
                    usage(1);
                }

                crate_filter = Some((*crate_filter_arg).to_owned());
                cursor = rest;
            }
            [dir_arg, rest @ ..] => {
                if let Some(dir_cur) = directory {
                    eprintln!("multiple directories provided: '{dir_cur}' and '{dir_arg}'");
                    usage(1);
                }

                directory = Some((*dir_arg).to_owned());
                cursor = rest;
            }
            _ => break,
        }
    }

    let directory = abs_directory_or_cwd(directory);

    Args {
        directory,
        crate_filter,
    }
}

fn usage(retcode: i32) -> ! {
    let text = "\
Usage: getpanics [directory]

An optional argument [directory] specifies where the project is located. If
not provided, the current working directory is used instead.";

    println!("{text}");
    exit(retcode);
}

fn version() -> ! {
    println!("getpanics {}", env!("CARGO_PKG_VERSION"));
    exit(0);
}

fn abs_directory_or_cwd(directory: Option<String>) -> PathBuf {
    match directory {
        Some(dir) => std::fs::canonicalize(dir).or_die("canonicalize path"),
        None => std::env::current_dir().or_die("get current working directory"),
    }
}

pub fn choose_target(args: &Args, targets: &[ra::TargetData]) -> ra::TargetData {
    match targets {
        [] => die!("Error: no local packages found"),
        [target] => target.clone(),
        _ => {
            let Some(crate_filter) = &args.crate_filter else {
                println!(
                    "\
Error: multiple crates available. Use the `--crate <name>` option to specify which one to examine."
                );
                print_targets(targets);
            };

            for target in targets {
                if target.name.contains(crate_filter) {
                    return target.clone();
                }
            }

            println!("Error: no matching crate found");
            print_targets(targets);
        }
    }
}

fn print_targets(targets: &[ra::TargetData]) -> ! {
    println!("Available crates:");

    for target in targets {
        let name = &target.name;
        let desc = utils::describe_target_kind(target.kind);

        println!("    {name} ({desc})");
    }

    exit(1);
}
