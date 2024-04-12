use std::fmt::Display;

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
