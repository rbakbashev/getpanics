use std::process::exit;

pub struct Args {
    pub directory: String,
}

pub fn parse() -> Args {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let refs = args.iter().map(String::as_str).collect::<Vec<_>>();

    let mut cursor = refs.as_slice();
    let mut directory = None;

    loop {
        match cursor {
            ["-h" | "--help", ..] => usage(0),
            ["-v" | "--version", ..] => version(),
            [dir_arg, rest @ ..] => {
                if let Some(dir_cur) = directory {
                    eprintln!("Error: multiple directories provided: '{dir_cur}' and '{dir_arg}'");
                    usage(1);
                }

                directory = Some((*dir_arg).to_owned());
                cursor = rest;
            }
            _ => break,
        }
    }

    let directory = directory.unwrap_or_else(|| ".".to_owned());

    Args { directory }
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
