//! Fuzz tester utility.

use std::io::{self, Write};
use std::mem;
use std::thread;
use std::time::Duration;

use color_print::cprintln;

use clap::Parser;

use bit_vec::BitVec;

use runner::{ClientProc, KvResp, RunnerError};

// Hardcoded constants:
const KEY_LEN: usize = 8;
const VALUE_LEN: usize = 16;
const RESP_TIMEOUT: Duration = Duration::from_secs(60);
const REMAIN_THRESH: usize = 1000;

mod random;
use random::*;

mod history;
use history::*;

/// Statistics about the fuzz testing round.
struct Stats {
    cnt_put: usize,
    cnt_swap: usize,
    cnt_get: usize,
    cnt_scan: usize,
    cnt_delete: usize,
    keys_freq: Vec<Vec<usize>>,
}

impl Stats {
    fn new(keys: &[Vec<String>]) -> Self {
        Stats {
            cnt_put: 0,
            cnt_swap: 0,
            cnt_get: 0,
            cnt_scan: 0,
            cnt_delete: 0,
            keys_freq: keys.iter().map(|ks| vec![0; ks.len()]).collect(),
        }
    }

    fn print(&self) {
        println!(
            "  Ops stats:  Put {}  Swap {}  Get {}  Scan {}  Delete {}",
            self.cnt_put, self.cnt_swap, self.cnt_get, self.cnt_scan, self.cnt_delete
        );
        for i in 0..self.keys_freq.len() {
            if i == 0 {
                print!("  Keys freq:  ");
            } else {
                print!("              ");
            }
            println!("{:?}", self.keys_freq[i]);
        }
    }
}

/// Fuzz testing logic, returning true if passed, else false. Returns the
/// number of pending checks in the check queue upon seemingly successful
/// test, or returns `None` if the test failed explicitly.
fn fuzz_test(
    args: &Args,
    keys: &[Vec<String>],
    stats: &mut Stats,
    mut clients: Vec<ClientProc>,
) -> Result<Option<usize>, RunnerError> {
    // use a bitmap to track which clients have on-the-fly requests; the fuzzer
    // randomly attempts to issue a new request or harvest a new response,
    // forcing the latter if all clients have on-the-fly requests
    let mut flying = BitVec::from_elem(clients.len(), false);

    // use a monotonically increasing logical timestamp counter as the "physical"
    // timestamps of client requests
    let mut timestamp = 0;
    let mut call_memo = vec![(0, None); clients.len()]; // (start_ts, put_value)

    // per-key per-client update history for consistency checking
    let mut history = History::new(clients.len(), keys);

    let total_ops = args.num_ops * args.num_clis;
    let mut ops_called = 0;
    let mut ops_waited = 0;
    let mut passed = true;
    let mut fuzzer_stdout = io::stdout();

    while ops_waited < total_ops {
        timestamp += 1;

        if gen_call_vs_wait(ops_called, total_ops, &flying) {
            // make a new call
            let cidx = gen_rand_client(&flying, false);

            let call = gen_rand_kvcall(&keys[cidx], stats, cidx);
            call_memo[cidx] = (timestamp, call.update_info());

            // eprintln!("calling {:?} @ {}", call, timestamp);
            clients[cidx].send_call(call)?;
            // eprintln!("called");

            flying.set(cidx, true);
            ops_called += 1;
        } else {
            // harvest a response
            let cidx = gen_rand_client(&flying, true);

            // RESP_TIMEOUT should be long enough to prevent false negatives
            // eprintln!("waiting");
            let resp = clients[cidx].wait_resp(RESP_TIMEOUT)?;
            // eprintln!("waited {:?} @ {}", resp, timestamp);
            if let KvResp::Stop = resp {
                cprintln!(
                    "<s><red>Unexpected stop response:</></>  client {}  <<{}>>",
                    cidx,
                    timestamp
                );
                passed = false;
                break;
            }

            // add to consistency violation check queue
            let (ts_call, update_info) = mem::take(&mut call_memo[cidx]);
            let ts_resp = timestamp;
            history.add_to_queue(ts_call, ts_resp, resp);

            // if is an update action, add to the update history, possibly
            // triggering some pending checks
            if let Some((update_key, update_value)) = update_info {
                match history.apply_update(cidx, ts_call, ts_resp, update_key, update_value) {
                    Some(Some(resp)) => {
                        cprintln!(
                            "<s><red>Consistency violation!</></>  Trigger:  client {}  <<{} - {}>>",
                            cidx,
                            ts_call,
                            ts_resp
                        );
                        println!("  Resp: {:?}", resp);
                        passed = false;
                        break;
                    }
                    Some(None) => {
                        cprintln!(
                            "<s><red>Unexpected update key found:</></>  client {}  <<{} - {}>>",
                            cidx,
                            ts_call,
                            ts_resp
                        );
                        passed = false;
                        break;
                    }
                    None => {}
                }
            }

            flying.set(cidx, false);
            ops_waited += 1;

            // progress printing
            if ops_waited % (total_ops / 100) == 0 || ops_waited == total_ops {
                print!(
                    "  Progress:  called {} / {}  waited {} / {}\r",
                    ops_called, total_ops, ops_waited, total_ops
                );
                fuzzer_stdout.flush()?;
                if ops_waited == total_ops {
                    println!();
                }
            }
        }
    }

    if passed {
        cprintln!("<s><yellow>Stopping clients...</></>");
        for client in clients {
            client.stop()?;
        }
    }
    Ok(if passed {
        Some(history.queue_len())
    } else {
        None
    })
}

/// Fuzzer utility arguments.
#[derive(Parser, Debug)]
struct Args {
    /// Number of concurrent clients.
    #[arg(long, default_value = "1")]
    num_clis: usize,

    /// Number of keys touched by each client.
    #[arg(long, default_value = "5")]
    num_keys: usize,

    /// Average number of operations per client to run.
    #[arg(long, default_value = "5000")]
    num_ops: usize,

    /// False if use disjoint sets of keys per client, otherwise true.
    #[arg(long, default_value = "false")]
    conflict: bool,

    /// Client `just` invocation arguments.
    #[arg(long, num_args(1..))]
    client_just_args: Vec<String>,
}

fn main() -> Result<(), RunnerError> {
    let args = Args::parse();
    cprintln!("<s><yellow>Fuzz testing configuration:</></> {:#?}", args);
    assert_ne!(args.num_clis, 0);
    assert_ne!(args.num_keys, 0);
    assert!(args.num_keys < 100000);
    assert!(args.num_ops >= 1000);

    // generate proper pool of keys
    let keys: Vec<Vec<String>> = (0..args.num_clis)
        .map(|_| {
            (0..args.num_keys)
                .map(|i| {
                    if args.conflict {
                        format!("key{:0w$}", i, w = KEY_LEN - 3)
                    } else {
                        gen_rand_string(KEY_LEN)
                    }
                })
                .collect()
        })
        .collect();
    let mut stats = Stats::new(&keys);

    // run clients concurrently
    let mut clients = vec![];
    for _ in 0..args.num_clis {
        let client = ClientProc::new(args.client_just_args.iter().map(|s| s.as_str()).collect())?;
        clients.push(client);
    }

    // wait for a few seconds to let cargo finish build check
    thread::sleep(Duration::from_secs(
        (0.3 * args.num_clis as f64).ceil() as u64
    ));
    cprintln!("<s><yellow>Fuzzing starts...</></>");

    // run fuzz testing
    let result = fuzz_test(&args, &keys, &mut stats, clients)?;
    stats.print();

    if let Some(remaining) = result {
        if remaining >= REMAIN_THRESH {
            // too many remaining checks, meaning some clients were not
            // completing requests and were lagging too much behind others
            cprintln!("<s><yellow>Fuzz testing result:</></> <red>UNFAIR</>");
            println!(
                "  Remaining checks queued:  {}  <s><red>too many!</></>",
                remaining
            );
        } else {
            // test (approximately) passed
            cprintln!("<s><yellow>Fuzz testing result:</></> <green>PASSED</>");
            println!("  Remaining checks queued:  {}  reasonable", remaining);
        }
    } else {
        // some check failed explicitly
        cprintln!("<s><yellow>Fuzz testing result:</></> <red>FAILED</>");
        println!("  Some check failed explicitly :-(");
    }
    Ok(())
}
