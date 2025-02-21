//! YCSB benchmarking utility.

use std::collections::{BTreeSet, HashMap};
use std::thread;
use std::time::Duration;

use color_print::cprintln;

use clap::Parser;

use runner::{ClientProc, RunnerError};

// Hardcoded constants:
const VALID_WORKLOADS: [char; 6] = ['a', 'b', 'c', 'd', 'e', 'f'];
const RESP_TIMEOUT: Duration = Duration::from_secs(60);
const YCSB_TIMEOUT: Duration = Duration::from_secs(600);

mod ycsb;
use ycsb::*;

/// Per-client performance statistics recording.
struct Stats {
    /// Number of client stats merged into this struct.
    merged: usize,
    // Performance statistics:
    total_ms: f64,                   // in millisecs
    tput_all: f64,                   // in ops/sec
    num_ops: HashMap<String, usize>, // map from op type -> count
    lat_avg: HashMap<String, f64>,   // map from op type -> microsecs
    lat_min: HashMap<String, f64>,
    lat_max: HashMap<String, f64>,
    lat_p99: HashMap<String, f64>,
}

impl Stats {
    fn new() -> Self {
        Stats {
            merged: 0,
            total_ms: 0.0,
            tput_all: 0.0,
            num_ops: HashMap::new(),
            lat_avg: HashMap::new(),
            lat_min: HashMap::new(),
            lat_max: HashMap::new(),
            lat_p99: HashMap::new(),
        }
    }

    fn print(&self, phase: &str) {
        cprintln!(
            "  <cyan>{:6}</>  {:6.0} ms",
            format!("[{}]", phase),
            self.total_ms
        );
        println!("    Throughput:  {:9.2} ops/sec", self.tput_all);

        for (i, op) in self.lat_avg.keys().enumerate() {
            debug_assert!(self.lat_min.contains_key(op));
            debug_assert!(self.lat_max.contains_key(op));
            debug_assert!(self.lat_p99.contains_key(op));
            if i == 0 {
                print!("    Latency:");
            } else {
                print!("            ");
            }
            println!(
                "    {:6}  ops {:6}  avg {:9.2}  min {:6.0}  max {:6.0}  p99 {:6.0}  us",
                op,
                self.num_ops[op],
                self.lat_avg[op],
                self.lat_min[op],
                self.lat_max[op],
                self.lat_p99[op]
            );
        }
    }

    /// Merge with another stats struct, taking reasonable arithmetics on the
    /// stats fields.
    fn merge(&mut self, other: Stats) {
        debug_assert_ne!(other.merged, 0);
        if self.merged == 0 {
            *self = other;
        } else {
            // take longer total run time
            self.total_ms = f64::max(self.total_ms, other.total_ms);
            // take sum of throughput
            self.tput_all += other.tput_all;
            // take sum of operations count
            Self::merge_map(&mut self.num_ops, other.num_ops, |sc, oc| sc + oc);
            // take overall average of avg latency
            Self::merge_map(&mut self.lat_avg, other.lat_avg, |sl, ol| {
                (sl * self.merged as f64 + ol * other.merged as f64)
                    / (self.merged + other.merged) as f64
            });
            // take min/max of min/max latency
            Self::merge_map(&mut self.lat_min, other.lat_min, f64::min);
            Self::merge_map(&mut self.lat_max, other.lat_max, f64::max);
            // for P99 latency, take the max, which should be reasonable
            Self::merge_map(&mut self.lat_p99, other.lat_p99, f64::max);
        }
    }

    /// Internal helper for the merge of op count or latency values.
    fn merge_map<T, F>(
        self_lat: &mut HashMap<String, T>,
        other_lat: HashMap<String, T>,
        merge_fn: F,
    ) where
        T: Copy,
        F: Fn(T, T) -> T,
    {
        for (op, self_lat) in &mut *self_lat {
            if let Some(other_lat) = other_lat.get(op) {
                *self_lat = merge_fn(*self_lat, *other_lat);
            }
        }
        for (op, other_lat) in other_lat {
            self_lat.entry(op).or_insert(other_lat);
        }
    }
}

/// YCSB benchmarking logic.
fn ycsb_bench(
    args: &Args,
    clients: Vec<ClientProc>,
    load: bool,
    ikeys: BTreeSet<String>,
) -> Result<(Stats, BTreeSet<String>), RunnerError> {
    let mut drivers = vec![];
    for client in clients {
        drivers.push(YcsbDriver::exec(
            args.workload,
            args.num_ops,
            load,
            client,
            ikeys.clone(),
        )?);
    }
    println!("  Launched {} YCSB drivers, now waiting...", drivers.len());

    let mut stats = Stats::new();
    let mut ikeys = BTreeSet::new();
    for driver in drivers {
        if let Some((cli_stats, cli_ikeys)) = driver.wait(YCSB_TIMEOUT)? {
            stats.merge(cli_stats);
            ikeys.extend(cli_ikeys);
        } else {
            return Err(RunnerError::Io("a YCSB driver process failed".into()));
        }
    }

    Ok((stats, ikeys))
}

/// Launcher utility arguments.
#[derive(Parser, Debug)]
struct Args {
    /// Number of concurrent clients.
    #[arg(long, default_value = "1")]
    num_clis: usize,

    /// Number of operations per client to run.
    #[arg(long, default_value = "10000")]
    num_ops: usize,

    /// YCSB workload profile name ('a' to 'f').
    #[arg(long, default_value = "a")]
    workload: char,

    /// Client `just` invocation arguments.
    #[arg(long, num_args(1..))]
    client_just_args: Vec<String>,
}

fn main() -> Result<(), RunnerError> {
    let args = Args::parse();
    cprintln!("<s><yellow>YCSB benchmark configuration:</></> {:#?}", args);
    assert_ne!(args.num_clis, 0);
    assert!(VALID_WORKLOADS.contains(&args.workload));

    // YCSB benchmark load phase
    let (stats_load, ikeys_load) = {
        // run load-phase clients concurrently
        let mut clients_load = vec![];
        for _ in 0..args.num_clis {
            let client =
                ClientProc::new(args.client_just_args.iter().map(|s| s.as_str()).collect())?;
            clients_load.push(client);
        }

        // wait for a few seconds to let cargo finish build check
        thread::sleep(Duration::from_secs(
            (0.3 * args.num_clis as f64).ceil() as u64
        ));
        cprintln!("<s><yellow>Benchmarking [Load] phase...</></>");

        ycsb_bench(&args, clients_load, true, BTreeSet::new())?
    };

    // YCSB benchmark run phase
    let (stats_run, _) = {
        // run run-phase clients concurrently
        let mut clients_run = vec![];
        for _ in 0..args.num_clis {
            let client =
                ClientProc::new(args.client_just_args.iter().map(|s| s.as_str()).collect())?;
            clients_run.push(client);
        }

        // wait for a few seconds to let cargo finish build check
        thread::sleep(Duration::from_secs(
            (0.3 * args.num_clis as f64).ceil() as u64
        ));
        cprintln!("<s><yellow>Benchmarking [Run] phase...</></>");

        ycsb_bench(&args, clients_run, false, ikeys_load)?
    };

    cprintln!(
        "<s><yellow>Benchmarking results:</></>  <cyan>YCSB-{}</>  <magenta>{} clients</>",
        args.workload,
        args.num_clis
    );
    stats_load.print("Load");
    stats_run.print("Run");
    Ok(())
}
