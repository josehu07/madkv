//! Driver library that runs a key-value client with automated workloads.

// TODO:
//   ycsb benchmarker
//   report generator?
//   web vis frontend?

mod error;
pub use error::RunnerError;

mod ioapi;
pub use ioapi::{KvCall, KvResp};

mod proc;
pub use proc::{ClientProc, ServerProc};
