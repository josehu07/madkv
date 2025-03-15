use std::collections::HashSet;
use std::process::{Child, Command};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use std::thread;
use std::time::{Duration, SystemTime};

use clap::Parser;
use itertools::izip;
use rand::Rng;

#[derive(Clone)]
struct Env {
    username: String,
    path: String,
}

fn setup_server(
    server_id: i32,
    ip: &str,
    disk_id: i32,
    port: i32,
    manager: &(String, i32),
    env: &Env,
) -> Child {
    let command = format!(
        r#"
echo $$ >> /tmp/madkv-run-p2-pids
cd {5}
sudo ./scripts/setup_fs.sh {0}
sudo ./scripts/map_mount.sh {0}
touch /mnt/madkv-{0}/MADKV_MARKER
while true; do
    if [[ -e /mnt/madkv-{0}/MADKV_MARKER ]]; then
        # still have race conditions
        # but not likely
        just p2::server {1} {2}:{3} {4} /mnt/madkv-{0}/log
    fi
    sleep 1
done"#,
        disk_id, server_id, manager.0, manager.1, port, env.path
    );
    Command::new("ssh")
        .arg(format!("{}@{}", env.username, ip))
        .arg(command)
        .spawn()
        .unwrap()
}

fn reset_disk(_server_id: i32, ip: &str, disk_id: i32, _port: i32, env: &Env) {
    let command = format!(
        r#"
cd {1}
sudo ./scripts/crash_map.sh {0}
sudo ./scripts/map_mount.sh {0}"#,
        disk_id, env.path
    );
    #[allow(clippy::zombie_processes)]
    let _ = Command::new("ssh")
        .arg(format!("{}@{}", env.username, ip))
        .arg(command)
        .spawn()
        .unwrap();
}

fn periodic_reset_disks(servers: &[(String, i32, i32)], stop: Arc<AtomicBool>, env: &Env) {
    let mut last_crash: Vec<Option<f64>> = vec![None; servers.len()];
    let mut rng = rand::rng();
    while !stop.load(Ordering::Relaxed) {
        let sleep_time = rng.random_range(1.0..2.0);
        thread::sleep(Duration::from_secs_f64(sleep_time));

        let server_index = rng.random_range(0..servers.len());
        let (ref ip, disk_id, port) = servers[server_index];
        let curr_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .expect("Time went backwards")
            .as_secs_f64();

        if last_crash[server_index].is_none_or(|t| t + 5.0 < curr_time) {
            println!("inject fault on server {}", server_index);
            last_crash[server_index] = Some(curr_time);
            reset_disk(server_index as i32, ip, disk_id, port, env);
        }
    }
}

fn run_manager(manager: &(String, i32), servers: &[(String, i32, i32)], env: &Env) {
    let manager_ip = &manager.0;
    let manager_port = manager.1;
    let servers_str: Vec<String> = servers
        .iter()
        .map(|(ip, _disk, port)| format!("{}:{}", ip, port))
        .collect();
    let servers_str = servers_str.join(",");
    let command = format!(
        r#"
cd {0}
just p2::manager {manager_port} {servers_str}
"#,
        env.path
    );
    #[allow(clippy::zombie_processes)]
    let _ = Command::new("ssh")
        .arg(format!("{}@{}", env.username, manager_ip))
        .arg(command)
        .spawn()
        .unwrap();
}

fn kill_all(servers: &[(String, i32, i32)], env: &Env) {
    let unique_ips: HashSet<&String> = servers.iter().map(|(ip, _, _)| ip).collect();
    for ip in unique_ips {
        let command = format!(
            r#"
cd {}
just p2::kill
if [ -e /tmp/madkv-run-p2-pids ]; then
    while read -r pid; do
        if [ -n "$pid" ]; then
            pkill -P "$pid"
            kill "$pid"
        fi
    done < /tmp/madkv-run-p2-pids
    rm /tmp/madkv-run-p2-pids
fi
"#,
            env.path
        );
        let _ = Command::new("ssh")
            .arg(format!("{}@{}", env.username, ip))
            .arg(command)
            .status()
            .expect("failed to run kill_all command");
    }
}

fn run(
    cmd: &str,
    servers: Vec<(String, i32, i32)>,
    manager: &(String, i32),
    crash: bool,
    env: &Env,
) {
    kill_all(&servers, env);
    run_manager(manager, &servers, env);

    let mut server_procs: Vec<Child> = Vec::new();
    for (server_id, (ip, disk_id, port)) in servers.iter().enumerate() {
        let child = setup_server(server_id as i32, ip, *disk_id, *port, manager, env);
        server_procs.push(child);
    }
    thread::sleep(Duration::from_secs(5));

    let command = format!(
        r#"
cd {}
just {} {}:{}
"#,
        env.path, cmd, manager.0, manager.1
    );
    let mut fuzz_proc = Command::new("sh")
        .arg("-c")
        .arg(command)
        .spawn()
        .expect("failed to spawn fuzz process");

    let stop = Arc::new(AtomicBool::new(false));
    let crash_handle = if crash {
        let servers_clone = servers.clone();
        let stop_clone = Arc::clone(&stop);
        let env = env.clone();
        Some(thread::spawn(move || {
            periodic_reset_disks(&servers_clone, stop_clone, &env);
        }))
    } else {
        None
    };

    fuzz_proc.wait().expect("failed waiting for fuzz process");
    kill_all(&servers, env);

    if crash {
        stop.store(true, Ordering::Relaxed);
        if let Some(handle) = crash_handle {
            handle.join().expect("failed to join crash thread");
        }
    }

    for mut proc in server_procs {
        let _ = proc.kill();
    }
}

#[derive(Parser, Debug)]
/// Run all experiments required for Project 2.
/// Do NOT run the servers on your personal machine unless you know what you're doing!
/// Use `--help` to see the long help message.
///
/// # Setup
///
/// This script assumes that you are using one NVME SSD for each server instance.
/// You can spread those instances across multiple physical machines, but the total number of
/// instances must be 5.
/// The manager always runs on the first machine specified.
/// Make sure you can ssh into all servers with ssh keys.
///
/// Before running each experiment, the script sets up a clean ext4 file system with
/// [fast_commit](https://dl.acm.org/doi/abs/10.5555/3691992.3692002) turned on, which is suitable
/// for fsync-heavy workloads.
/// Since this needs root privilege, it assumes you can run `sudo` without a password.
///
/// # Fault injection
///
/// When running the fuzz tester with crashing servers, the script periodically chooses a server
/// and simulates a disk disconnection on it. This is achieved via the
/// [Linux device mapper](https://docs.kernel.org/admin-guide/device-mapper/index.html).
/// Your server should experience I/O errors and may crash. New server instances will be restarted
/// once the script sets up the file system again.
///
/// # Example
///
/// ```bash
/// cargo run --bin run_p2 -- \
///     --username ljx --path "~/739madkv/" \
///     --address 128.105.146.89 \
///     --num-server 3 \
///     --disk-id-start 1 \
///     --address 128.105.146.88 \
///     --num-server 2 \
///     --disk-id-start 1
/// ```
///
/// With two Cloudlab sm110p machines, this typically takes around 10 minutes.
struct Args {
    #[arg(long)]
    username: String,
    #[arg(long)]
    path: String,
    #[arg(long, required = true)]
    /// These three args can be repeated. Matched by the order they appear.
    address: Vec<String>,
    #[arg(long, required = true)]
    /// Number of server instances on this **single** machine.
    num_server: Vec<i32>,
    #[arg(long, required = true)]
    /// The first disk to use for this machine. You might want to skip the ones you use for other
    /// purposes.
    /// e.g. if num_server=2, disk_id_start=1 means /dev/nvme1n1 and /dev/nvme2n1 are used.
    disk_id_start: Vec<i32>,
}

fn main() {
    let args = Args::parse();
    let env = Env {
        username: args.username,
        path: args.path,
    };
    assert_eq!(args.address.len(), args.num_server.len());
    assert_eq!(args.address.len(), args.disk_id_start.len());
    assert_eq!(args.num_server.iter().sum::<i32>(), 5);

    let servers: Vec<(String, i32, i32)> =
        izip!(&args.address, args.num_server, args.disk_id_start)
            .map(move |(address, num_server, disk_id_start)| {
                (disk_id_start..disk_id_start + num_server)
                    .map(move |i| (address.clone(), i, 3700 + i))
            })
            .flatten()
            .collect();
    let manager = (args.address.first().unwrap().clone(), 3777);

    let fuzz = |num_server: usize, crash: bool| {
        let crash_str = if crash { "yes" } else { "no" };
        run(
            &format!("p2::fuzz {num_server} {crash_str}"),
            servers[..num_server].to_vec(),
            &manager,
            crash,
            &env,
        );
    };

    let bench = |num_server: usize, num_client: usize, workload: char| {
        run(
            &format!("p2::bench {num_client} {workload} {num_server}"),
            servers[..num_server].to_vec(),
            &manager,
            false,
            &env,
        );
    };

    fuzz(3, false);
    fuzz(3, true);
    fuzz(5, true);

    for workload in 'a'..='f' {
        for num_server in [1, 3, 5] {
            bench(num_server, 10, workload);
        }
    }

    for num_client in [1, 20, 30] {
        for num_server in [1, 5] {
            bench(num_server, num_client, 'a');
        }
    }
}
