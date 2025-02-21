//! Unified error type for easier error handling.

use std::error::Error;
use std::fmt;
use std::io;
use std::num;
use std::str;
use std::sync::mpsc;

/// Unified error type in the runner utility.
#[derive(Debug, Clone)]
pub enum RunnerError {
    Io(String),
    Parse(String),
    Chan(String),
    Join,
}

impl Error for RunnerError {}

impl fmt::Display for RunnerError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            RunnerError::Io(msg) => write!(f, "io error: {}", msg),
            RunnerError::Parse(msg) => write!(f, "parse error: {}", msg),
            RunnerError::Chan(msg) => write!(f, "chan error: {}", msg),
            RunnerError::Join => write!(f, "thread join error"),
        }
    }
}

/// Convenient macro to implement `From<>` traits for `RunnerError`.
macro_rules! impl_from {
    ($fromtype:ty, $variant:ident) => {
        impl From<$fromtype> for RunnerError {
            fn from(err: $fromtype) -> RunnerError {
                RunnerError::$variant(err.to_string())
            }
        }
    };
}

/// Single-generic form of the `impl_from!` macro.
macro_rules! impl_from_generic {
    ($fromtype:ty, $variant:ident) => {
        impl<T> From<$fromtype> for RunnerError {
            fn from(err: $fromtype) -> RunnerError {
                RunnerError::$variant(err.to_string())
            }
        }
    };
}

impl_from!(io::Error, Io);
impl_from!(str::ParseBoolError, Parse);
impl_from!(num::ParseIntError, Parse);
impl_from!(num::ParseFloatError, Parse);
impl_from!(mpsc::RecvError, Chan);
impl_from!(mpsc::RecvTimeoutError, Chan);

impl_from_generic!(mpsc::SendError<T>, Chan);
