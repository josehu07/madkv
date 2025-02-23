# MadKV

![top-lang](https://img.shields.io/github/languages/top/josehu07/madkv?color=darkorange)
![code-size](https://img.shields.io/github/languages/code-size/josehu07/madkv?color=steelblue)
![license](https://img.shields.io/github/license/josehu07/madkv?color=green)

This is the distributed key-value (KV) store project template for the Distributed Systems course (CS 739) at the University of Wisconsin--Madison. Through a few steps over the semester, students will build MadKV, a replicated, partitioned, consensus-backed, fault-tolerant, performant, and extensible key-value store system in any language of their choice.

Recommended project steps over a semester:

1. Client/Server Key-Value Store Basics
2. Durable Storage & Keyspace Partitioning
3. Linearizable Replication w/ Consensus
4. Open-ended project that extends the system

To get started, clone the repo to your development machines:

```bash
git clone https://github.com/josehu07/madkv.git
```

The codebase template is subject to updates between project releases. Pull and merge updates into your development branch before working on a new project.

```bash
git checkout main
git pull
git checkout proj
git merge main
```

## Prerequisites

Install the following dependencies on all machines (instructions are for CloudLab instances running Ubuntu 22.04):

<details>
<summary>Rust toolchain (>= 1.84)...</summary>
<p></p>

```bash
# rustc & cargo, etc.
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

</details>

<details>
<summary>Python 3.12 & libraries...</summary>
<p></p>

```bash
# uv manager
curl -LsSf https://astral.sh/uv/install.sh | sh
source $HOME/.local/bin/env

# python 3.12 & fetch deps
cd madkv
uv python install 3.12
uv sync
```

</details>

<details>
<summary>Packages (tree, just >= 1.34, java)...</summary>
<p></p>

```bash
# add just gpg
wget -qO - 'https://proget.makedeb.org/debian-feeds/prebuilt-mpr.pub' | gpg --dearmor | sudo tee /usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg 1> /dev/null
echo "deb [arch=all,$(dpkg --print-architecture) signed-by=/usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg] https://proget.makedeb.org prebuilt-mpr $(lsb_release -cs)" | sudo tee /etc/apt/sources.list.d/prebuilt-mpr.list

# apt install packages
sudo apt update
sudo apt install tree just default-jre liblog4j2-java
```

</details>

## Code Structure

The codebase contains the following essential files:

* `Justfile`: the top-level Justfile, the entrance to `just` invocations
* `justmod/`: project-specific Justfiles to be included as modules
* `refcli/`: a dummy client that demonstrates the stdin/out workloads interface
* `runner/`: a multi-functional KV testing & benchmarking utility
* `sumgen/`: helper scripts for plotting & report generation
* `kvstore/` or any other directory name to your liking: source code of your KV store components

Students will implement their KV store server, clients, and other components under some subdirectory (e.g., `kvstore/`) in any language/framework of their choice, and add proper invocation commands to project-specific Justfiles for automation. We recommend students get familiar with the basics of the [`just` tool](https://github.com/casey/just).

See the course Canvas specs for details about the KV store projects and the tasks to complete. Our unoptimized reference solution at the end of Project 3 stands at ~2.5k lines of async Rust, not counting unit tests.

## Just Recipes

<details>
<summary>The following are common just recipes (subject to updates)...</summary>
<p></p>

List `just` recipes (of a module):

```bash
just [module]
```

List all files in the codebase as a tree:

```bash
just tree
```

Build or clean the provided utilities:

```bash
just utils::build
just utils::clean
```

Fetch the YCSB benchmark to `ycsb/`:

```bash
just utils::ycsb
```

</details>

All actions relevant to grading should be made invocable through `just` recipes. Students need to fill out some of the recipes in the project-level `Justfile`s (e.g., `justmod/proj1.just`) to surface their own KV store system code.

For each project, fill in the blanks of `justmod/proj<x>.just` with proper commands to invoke your KV server and client executables. Then, follow the Canvas project spec and complete the required tasks.

### Project 1

<details>
<summary>The following recipes should be ready for project 1...</summary>
<p></p>

Install extra dependencies of your KV system code if any (e.g., protobuf compiler):

```bash
just p1::deps
```

Build or clean your KV store executables:

```bash
just p1::build
just p1::clean
```

Launch the KV store server process, listening on address:

```bash
just p1::server <listen_addr>
```

Run a KV store client process in stdin/out workload automation mode, connecting to server at address:

```bash
just p1::client <server_addr>
```

Run a student-provided testcase demonstration client:

```bash
just p1::test<n> <server_addr>
```

Kill all processes relevant to your KV store system:

```bash
just p1::kill
```

Once these recipes are correctly supplied and properly tested, the following higher-level recipes will be runnable.

Launch the long-running KV store server:

```bash
just p1::service <listen_addr>
```

Run a student-provided testcase and record outputs to `/tmp/madkv-p1/tests/`:

```bash
just p1::testcase <num> <server_addr>
```

Run fuzz testing with given configuration and record outputs to `/tmp/madkv-p1/fuzz/`:

```bash
just p1::fuzz <nclients> <conflict ("yes" or "no")> <server_addr>
```

Run YCSB benchmarking with given configuration and record outputs to `/tmp/madkv-p1/bench/`:

```bash
just p1::bench <nclients> <workload ("a" to "f")> <server_addr>
```

Generate a report template at `report/proj1.md` from saved results under `/tmp/madkv-p1/`:

```bash
just p1::report
```

This command first prints a list of testing & benchmarking configurations you need to run and get outputs. Once all outputs are ready under `/tmp/madkv-p1/`, it generates the report template and plots selected performance results. Download the `report/` directory (which includes generated plots) and make your edits to the report.

</details>

### Project 2

<details>
<summary>The following recipes should be ready for project 2...</summary>
<p></p>

Install extra dependencies of your KV system code if any (e.g., protobuf compiler, local storage library):

```bash
just p2::deps
```

Build or clean your KV store executables:

```bash
just p2::build
just p2::clean
```

Launch the KV store manager process, listening on `0.0.0.0:<man_port>` and expecting the given comma-separated list of servers to form the cluster:

```bash
just p2::manager <man_port> <pub_ip0>:<api_port0>,<pub_ip1>:<api_port1>,...
```

Launch a KV store server process with node ID `<id>`, connecting to manager at `<manager_addr>` to register and listening on `0.0.0.0:<api_port>` for clients, using `<backer_path>` directory for durable storage:

```bash
just p2::server <id> <manager_addr> <api_port> <backer_path>
```

Run a KV store client process in stdin/out workload automation mode, connecting to manager at address:

```bash
just p2::client <manager_addr>
```

Kill all processes relevant to your KV store system:

```bash
just p2::kill
```

Once these recipes are correctly supplied, the following higher-level recipes will be runnable.

Launch the KV store service components that reside in node `<node_id>`. This recipe uses the following convention:

* manager uses the node ID `m`, and listens on `0.0.0.0:<man_port>`
* server `x` uses the node ID `sx` where `x` is a partition ID integer (e.g., `s0`, `s1`, etc.). Server connects to manager using address `<man_ip>:<man_port>` and listens on `0.0.0.0:<api_portx>` for clients, and uses `<backer_prefix>.<node_id>/` as the durable storage path

```bash
just p2::service <node_id> <man_ip>:<man_port> <pub_ip0>:<api_port0>,<pub_ip1>:<api_port1>,... <backer_prefix>
```

The `service` recipe needs to be run for all server nodes (with the proper node ID changed, but all other arguments kept the same) to establish the KV service.

Run fuzz testing and record outputs to `/tmp/madkv-p2/fuzz/`. This time we always use 5 clients with conflicting keys. The parameters `<nservers>` and `<crashing>` are only used in setting the output log's filename; service behavior should be controlled manually:

```bash
just p2::fuzz <nservers> <crashing ("no" or "yes")> <manager_addr>
```

Run YCSB benchmarking with given configuration and record outputs to `/tmp/madkv-p2/bench/`:

```bash
just p2::bench <nclients> <workload ("a" to "f")> <nservers> <manager_addr>
```

Generate a report template at `report/proj2.md` from saved results under `/tmp/madkv-p2/`:

```bash
just p2::report
```

This command first prints a list of testing & benchmarking configurations you need to run and get outputs. Once all outputs are ready under `/tmp/madkv-p2/`, it generates the report template and plots selected performance results. Download the `report/` directory (which includes generated plots) and make your edits to the report.

</details>

### Project 3

<details>
<summary>The following recipes should be ready for project 3...</summary>
<p></p>

Install extra dependencies of your KV system code if any (e.g., protobuf compiler, local storage library):

```bash
just p3::deps
```

Build or clean your KV store executables:

```bash
just p3::build
just p3::clean
```

Launch a KV store manager replica process (see Canvas spec for the format and interpretation of arguments):

```bash
just p3::manager <rep_id> <man_port> <p2p_port> <peer_addrs> <server_rf> <server_addrs>
```

Launch a KV store server replica process (see Canvas spec for the format and interpretation of arguments):

```bash
just p3::server <part_id> <rep_id> <manager_addrs> <api_port> <p2p_port> <peer_addrs> <backer_path>
```

Run a KV store client process in stdin/out workload automation mode, connecting to manager at address:

```bash
just p3::client <manager_addrs>
```

Kill all processes relevant to your KV store system:

```bash
just p3::kill
```

Once these recipes are correctly supplied, the following higher-level recipes will be runnable.

Launch the KV store service components that reside in node `<node_id>`. This recipe is getting messy in this project after replication is involved; it uses the following convention for its arguments:

* manager uses the node ID `m.x` where `x` is its replica ID, e.g., `m.0`, `m.1`, `m.2`, ...
* server `x.y` uses the node ID `sx.y` where `x` is its partition ID and `y` is its replica ID within that partition, e.g., `s0.0`. `s0.1`, `s0.2`, `s1.0`, ...
* `managers` is a comma-separated list of the manager replicas' public listen addresses
* `manager_p2ps` is a comma-separated list of the manager replicas' internal replication listen addresses
* `server_rf` is the server replication factor of a partition
* `servers` is a comma-separated list of the server nodes' public listen addresses, indexed first by partition then by replica (see Canvas spec for what this means)
* `server_p2ps` is a comma-separated list of the server nodes' internal replication listen addresses, indexed similarly to `servers`
* `backer_prefix` is the prefix of the path to durable storage directory

```bash
just p3::service <node_id> <managers> <manager_p2ps> <server_rf> <servers> <server_p2ps> <backer_prefix>
```

The `service` recipe needs to be run for all server nodes (with the proper node ID changed, but all other arguments kept the same) to establish the KV service.

Run fuzz testing and record outputs to `/tmp/madkv-p3/fuzz/`. This time we always use 5 clients with conflicting keys. The parameters `<nservers>` and `<crashing>` are only used in setting the output log's filename; service behavior should be controlled manually:

```bash
just p3::fuzz <server_rf> <crashing ("no" or "yes")> <manager_addrs>
```

Run YCSB benchmarking with given configuration and record outputs to `/tmp/madkv-p3/bench/`:

```bash
just p3::bench <nclients> <workload ("a" to "f")> <server_rf> <manager_addrs>
```

Generate a report template at `report/proj3.md` from saved results under `/tmp/madkv-p3/`:

```bash
just p3::report
```

This command first prints a list of testing & benchmarking configurations you need to run and get outputs. Once all outputs are ready under `/tmp/madkv-p3/`, it generates the report template and plots selected performance results. Download the `report/` directory (which includes generated plots) and make your edits to the report.

</details>

## Client Automation Interface

As described in the Canvas project specs, the KV client should by default support a stdin/out-based workload interface for automation. The interface is formatted as the following (but subject to updates across projects).

The client in this mode should block on reading stdin line by line, interpreting each line as a synchronous key-value API call.

<details>
<summary>The input lines have the following format...</summary>
<p></p>

```text
PUT <key> <value>
SWAP <key> <value>
GET <key>
DELETE <key>
SCAN <key123> <key456>
STOP  # stop reading stdin, exit
```

</details>

After the completion of an API call, the client should print to stdout a line (ending with a newline) that presents the result of this call.

<details>
<summary>The output lines should have the following format...</summary>
<p></p>

```text
PUT <key> found
PUT <key> not_found
SWAP <key> <old_value>
SWAP <key> null  # if not found
GET <key> <value>
GET <key> null   # if not found
DELETE <key> found
DELETE <key> not_found
SCAN <key123> <key456> BEGIN
  <key127> <valuea>
  <key299> <valueb>
  <key456> <valuec>
SCAN END
STOP  # confirm STOP before exit
```

</details>

Assume all keys and values are ASCII alphanumeric, case-sensitive strings. All keywords are also case-sensitive. All spaces are regular spaces and the number of them does not matter.

---

**PLEASE DO NOT FORK PUBLICLY OR PUBLISH SOLUTIONS ONLINE.**

Authored by [Guanzhou Hu](https://josehu.com). First offered in CS 739 Spring 2025 taught by [Prof. Andrea Arpaci-Dusseau](https://pages.cs.wisc.edu/~dusseau/). To get the associated project specs and a reference solution in Rust for teaching purposes, please contact us!

If you find replicated distributed systems interesting, take a look at [Summerset](https://github.com/josehu07/summerset) and [Linearize](https://github.com/josehu07/linearize) :-)
