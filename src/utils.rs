use std::fmt::Display;

use ra_ap_project_model::TargetKind;

#[macro_export]
macro_rules! die {
    ($( $arg:tt )*) => {{
        eprintln!($($arg)*);
        std::process::exit(1);
    }}
}

pub trait MaybeError<T> {
    #[track_caller]
    fn or_die(self, action: &str) -> T;
}

impl<T, E: Display> MaybeError<T> for Result<T, E> {
    fn or_die(self, action: &str) -> T {
        match self {
            Ok(t) => t,
            Err(e) => die!("Failed to {}: {}", action, e),
        }
    }
}

impl<T> MaybeError<T> for Option<T> {
    fn or_die(self, action: &str) -> T {
        match self {
            Some(t) => t,
            None => die!("Failed to {}", action),
        }
    }
}

pub fn describe_target_kind(kind: TargetKind) -> &'static str {
    match kind {
        TargetKind::Bin => "bin",
        TargetKind::Lib { is_proc_macro } => {
            if is_proc_macro {
                "proc-macro"
            } else {
                "lib"
            }
        }
        TargetKind::Example => "example",
        TargetKind::Test => "test",
        TargetKind::Bench => "bench",
        TargetKind::BuildScript => "build script",
        TargetKind::Other => "other",
    }
}
