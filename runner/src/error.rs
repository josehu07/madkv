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

impl From<io::Error> for RunnerError {
    fn from(err: io::Error) -> RunnerError {
        RunnerError::Io(err.to_string())
    }
}

impl From<str::ParseBoolError> for RunnerError {
    fn from(err: str::ParseBoolError) -> RunnerError {
        RunnerError::Parse(err.to_string())
    }
}

impl From<num::ParseIntError> for RunnerError {
    fn from(err: num::ParseIntError) -> RunnerError {
        RunnerError::Parse(err.to_string())
    }
}

impl From<num::ParseFloatError> for RunnerError {
    fn from(err: num::ParseFloatError) -> RunnerError {
        RunnerError::Parse(err.to_string())
    }
}

impl<T> From<mpsc::SendError<T>> for RunnerError {
    fn from(err: mpsc::SendError<T>) -> RunnerError {
        RunnerError::Chan(err.to_string())
    }
}

impl From<mpsc::RecvError> for RunnerError {
    fn from(err: mpsc::RecvError) -> RunnerError {
        RunnerError::Chan(err.to_string())
    }
}

impl From<mpsc::RecvTimeoutError> for RunnerError {
    fn from(err: mpsc::RecvTimeoutError) -> RunnerError {
        RunnerError::Chan(err.to_string())
    }
}
