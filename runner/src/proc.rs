//! Process running and management.

use std::io;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};

/// Wrapper handle to a KV server process.
pub(crate) struct ServerProc {
    handle: Child,
}

impl ServerProc {
    /// Run a server process using provided `just` recipe args, returning a
    /// handle to it.
    pub(crate) fn new(just_args: Vec<&str>) -> Result<ServerProc, io::Error> {
        let handle = Command::new("just").args(just_args).spawn()?;
        Ok(ServerProc { handle })
    }
}

/// Wrapper handle to a KV client process.
pub(crate) struct ClientProc {
    handle: Child,
    stdin: ChildStdin,
    stdout: ChildStdout,
}

impl ClientProc {
    /// Run a client process using provided `just` recipe args, returning a
    /// handle to it.
    pub(crate) fn new(just_args: Vec<&str>) -> Result<ClientProc, io::Error> {
        let mut handle = Command::new("just")
            .args(just_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;

        let stdin = handle.stdin.take().unwrap();
        let stdout = handle.stdout.take().unwrap();

        Ok(ClientProc {
            handle,
            stdin,
            stdout,
        })
    }
}
