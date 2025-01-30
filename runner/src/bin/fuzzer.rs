//! Fuzz tester utility.

use std::net::SocketAddr;

use clap::Parser;

use runner::{ClientProc, RunnerError, ServerProc};

/// Fuzzer utility arguments.
#[derive(Parser, Debug)]
struct Args {
    /// Server address.
    #[arg(short, long, default_value = "127.0.0.1:3777")]
    server: SocketAddr,

    /// Number concurrent clients.
    #[arg(short, long, default_value = "1")]
    num_clis: usize,

    /// Number of operations per client.
    #[arg(short, long, default_value = "10000")]
    num_ops: usize,

    /// False if use disjoint sets of keys per client, otherwise true.
    #[arg(short, long, default_value = "false")]
    conflict: bool,
}

fn main() -> Result<(), RunnerError> {
    let args = Args::parse();

    let mut clients = vec![];
    for _ in 0..args.num_clis {
        let client = ClientProc::new(vec!["client", args.server.to_string().as_str()])?;
        clients.push(client);
    }

    // TODO: impl me

    for client in clients {
        client.stop()?;
    }
    Ok(())
}
