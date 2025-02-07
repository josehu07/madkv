# MadKV

![top-lang](https://img.shields.io/github/languages/top/josehu07/madkv?color=darkorange)
![code-size](https://img.shields.io/github/languages/code-size/josehu07/madkv?color=steelblue)
![license](https://img.shields.io/github/license/josehu07/madkv?color=green)

This is the distributed key-value (KV) store project template for the Distributed Systems course (CS 739) at the University of Wisconsin--Madison. Through a few steps over the semester, students will build MadKV, a replicated, consensus-backed, fault-tolerant, and optionally partitioned key-value store system with good performance.

To get started, clone the repo to your development machine:

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
# pyenv
curl https://pyenv.run | bash
tee -a $HOME/.bashrc <<EOF
export PYENV_ROOT="\$HOME/.pyenv"
command -v pyenv >/dev/null || export PATH="\$PYENV_ROOT/bin:\$PATH"
eval "\$(pyenv init -)"
EOF
source $HOME/.bashrc

# python 3.12
sudo apt update
sudo apt install libssl-dev zlib1g-dev
pyenv install 3.12
pyenv global 3.12

# pip packages
pip3 install numpy matplotlib termcolor
```

</details>

<details>
<summary>Packages (tree, just >= 1.34, java)...</summary>
<p></p>

```bash
# add just gpg
wget -qO - 'https://proget.makedeb.org/debian-feeds/prebuilt-mpr.pub' | gpg --dearmor | sudo tee /usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg 1> /dev/null
echo "deb [arch=all,$(dpkg --print-architecture) signed-by=/usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg] https://proget.makedeb.org prebuilt-mpr $(lsb_release -cs)" | sudo tee /etc/apt/sources.list.d/prebuilt-mpr.list

# apt install
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
* `src/` or any other directory name to your liking: source code of your KV store server and client

Students will implement their KV store server and clients under some subdirectory (e.g., `src/`) in any language of their choice, and add proper invocation commands to project-specific Justfiles for automation. We recommend students get familiar with the basics of the [`just` tool](https://github.com/casey/just).

See the course Canvas specs for details about the KV store project and the tasks to complete.

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

Build the provided utilities:

```bash
just utils::build
```

Clean the build of provided utilities:

```bash
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

Once these recipes are correctly supplied, the following higher-level recipes will be runnable.

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
just p1::fuzz <num_clients> <conflict ("yes" or "no")> <server_addr>
```

Run YCSB benchmarking with given configuration and record outputs to `/tmp/madkv-p1/bench/`:

```bash
just p1::bench <num_clients> <workload ("a" to "f")> <server_addr>
```

Generate a report template at `report/proj1.md` from saved results under `/tmp/madkv-p1/`:

```bash
just p1::report
```

This command first prints a list of testing & benchmarking configurations you need to run and get outputs. Once all outputs are ready under `/tmp/madkv-p1/`, it generates the report template and plots selected performance results. Download the `report/` directory (which includes generated plots) and make your edits to the report.

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

Authored by [Guanzhou Hu](https://josehu.com). First offered in CS 739 Spring 2025 taught by [Prof. Andrea Arpaci-Dusseau](https://pages.cs.wisc.edu/~dusseau/).

If you find replicated distributed systems interesting, take a look at [Summerset](https://github.com/josehu07/summerset) and [Linearize](https://github.com/josehu07/linearize) :-)
