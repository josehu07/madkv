//! Process running and management.

use std::cell::RefCell;
use std::io::BufReader;
use std::process::{Child, ChildStdin, ChildStdout, Command, Stdio};
use std::sync::mpsc;
use std::thread::{self, JoinHandle};
use std::time::Duration;

use crate::{KvCall, KvResp, RunnerError};

thread_local! {
    /// Thread-local buffer for reading lines of client output.
    static READBUF: RefCell<String> = const { RefCell::new(String::new()) };
}

/// Wrapper handle to a KV server (or manager) process.
#[derive(Debug)]
pub struct ServerProc {
    handle: Child,
}

impl ServerProc {
    /// Run a server or manager process using provided `just` recipe args,
    /// returning a handle to it.
    pub fn new(just_args: Vec<&str>) -> Result<ServerProc, RunnerError> {
        let handle = Command::new("just").args(just_args).spawn()?;
        Ok(ServerProc { handle })
    }

    /// Wait for the server or manager process to exit (usually only happens
    /// on errors), returning `Ok` only upon successful termination.
    pub fn wait(mut self) -> Result<(), RunnerError> {
        let status = self.handle.wait()?;
        if status.success() {
            Ok(())
        } else {
            Err(RunnerError::Io(format!(
                "server exited with status: {}",
                status
            )))
        }
    }

    /// Kill the server or manager process, consuming self.
    pub fn stop(mut self) -> Result<(), RunnerError> {
        self.handle.kill()?;
        Ok(())
    }
}

/// Wrapper handle to a KV client process.
#[derive(Debug)]
pub struct ClientProc {
    handle: Child,
    _driver: JoinHandle<()>,
    call_tx: mpsc::Sender<KvCall>,
    resp_rx: mpsc::Receiver<KvResp>,
}

impl ClientProc {
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
    pub fn wait_resp(&mut self, timeout: Duration) -> Result<KvResp, RunnerError> {
        let resp = self.resp_rx.recv_timeout(timeout)?;
        Ok(resp)
    }

    /// Send stop to the client process (and kill it just to be sure),
    /// consuming self.
    pub fn stop(mut self) -> Result<(), RunnerError> {
        self.call_tx.send(KvCall::Stop)?;
        let resp = self.wait_resp(Duration::from_secs(10))?;
        if !matches!(resp, KvResp::Stop) {
            return Err(RunnerError::Io(
                "unexpected response, expecting STOP".into(),
            ));
        }

        self.handle.kill()?;
        Ok(())
    }

    /// One iteration of the driver thread loop.
    fn driver_iter(
        stdin: &mut ChildStdin,
        stdout: &mut BufReader<ChildStdout>,
        call_rx: &mpsc::Receiver<KvCall>,
        resp_tx: &mpsc::Sender<KvResp>,
        line: &mut String,
        stopped: &mut bool,
    ) -> Result<(), RunnerError> {
        let call = call_rx.recv()?;
        if let KvCall::Stop = call {
            *stopped = true;
        }
        call.into_write(stdin)?;

        let resp = KvResp::from_read(stdout, line)?;
        resp_tx.send(resp)?;
        Ok(())
    }

    /// Dedicated stdin/out API thread function.
    fn driver_thread(
        mut stdin: ChildStdin,
        stdout: ChildStdout,
        call_rx: mpsc::Receiver<KvCall>,
        resp_tx: mpsc::Sender<KvResp>,
    ) {
        READBUF.with(|buf| {
            let line = &mut buf.borrow_mut();
            let mut stdout = BufReader::new(stdout);
            let mut stopped = false;

            loop {
                if let Err(err) = Self::driver_iter(
                    &mut stdin,
                    &mut stdout,
                    &call_rx,
                    &resp_tx,
                    line,
                    &mut stopped,
                ) {
                    if !stopped && !matches!(err, RunnerError::Chan(_)) {
                        eprintln!("Error in driver: {}", err);
                    }
                    break;
                }
            }
        })
    }
}
