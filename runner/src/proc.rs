//! Process running and management.

use std::cell::RefCell;
use std::io::BufReader;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};

use crate::{KvCall, KvResp, RunnerError};

thread_local! {
    /// Thread-local buffer for reading lines of client output.
    static READBUF: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Wrapper handle to a KV server process.
#[derive(Debug)]
pub struct ServerProc {
    handle: Child,
}

impl ServerProc {
    /// Run a server process using provided `just` recipe args, returning a
    /// handle to it.
    pub fn new(just_args: Vec<&str>) -> Result<ServerProc, RunnerError> {
        let handle = Command::new("just").args(just_args).spawn()?;
        Ok(ServerProc { handle })
    }

    /// Kill the server process, consuming self.
    pub fn stop(mut self) -> Result<(), RunnerError> {
        self.handle.kill()?;
        Ok(())
    }
}

/// Wrapper handle to a KV client process.
#[derive(Debug)]
pub struct ClientProc {
    handle: Child,
    _driver: JoinHandle<Result<(), RunnerError>>,
    call_tx: mpsc::Sender<KvCall>,
    resp_rx: mpsc::Receiver<KvResp>,
}

impl ClientProc {
    /// Dedicated stdin/out API thread function.
    fn driver_thread(
        mut stdin: ChildStdin,
        stdout: ChildStdout,
        call_rx: mpsc::Receiver<KvCall>,
        resp_tx: mpsc::Sender<KvResp>,
    ) -> Result<(), RunnerError> {
        READBUF.with(|buf| {
            let line = &mut buf.borrow_mut();
            let mut stdout = BufReader::new(stdout);

            loop {
                let call = call_rx.recv()?;
                call.into_write(&mut stdin)?;
                let resp = KvResp::from_read(&mut stdout, line)?;
                resp_tx.send(resp)?;
            }
        })
    }

    /// Run a client process using provided `just` recipe args, returning a
    /// handle to it.
    pub fn new(just_args: Vec<&str>) -> Result<ClientProc, RunnerError> {
        let mut handle = Command::new("just")
            .args(just_args)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .spawn()?;
        let stdin = handle.stdin.take().unwrap();
        let stdout = handle.stdout.take().unwrap();

        // for each ClientProc initialized, there's a spawned thread for
        // handling the stdin/out workload API to and from the client process
        let (call_tx, call_rx) = mpsc::channel();
        let (resp_tx, resp_rx) = mpsc::channel();
        let driver = thread::spawn(move || Self::driver_thread(stdin, stdout, call_rx, resp_tx));

        Ok(ClientProc {
            handle,
            _driver: driver,
            call_tx,
            resp_rx,
        })
    }

    /// Send a KV operation call to the client process.
    pub fn send_call(&self, call: KvCall) -> Result<(), RunnerError> {
        self.call_tx.send(call)?;
        Ok(())
    }

    /// Wait for the next KV operation response from the client process.
    pub fn wait_resp(&mut self) -> Result<KvResp, RunnerError> {
        let resp = self.resp_rx.recv()?;
        Ok(resp)
    }

    /// Send stop to the client process (and kill it just to be sure),
    /// consuming self.
    pub fn stop(mut self) -> Result<(), RunnerError> {
        self.call_tx.send(KvCall::Stop)?;
        let resp = self.wait_resp()?;
        if !matches!(resp, KvResp::Stop) {
            return Err(RunnerError::Io(
                "unexpected response, expecting STOP".into(),
            ));
        }

        self.handle.kill()?;
        Ok(())
    }
}
