# MadKV

This is a distributed key-value store project template for the Distributed Systems course (CS 739) at the University of Wisconsin--Madison. Through a few steps over the semester, students will build MadKV, a replicated, consensus-backed, fault-tolerant, and optionally partitioned key-value store system with good performance.

## Prerequisites

Install the following dependencies (instructions are for CloudLab instances running Ubuntu 22.04):

* Rust toolchain (>= 1.84):

    ```bash
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
    ```

* Packages (`tree`, `just` >= 1.34):

    ```bash
    wget -qO - 'https://proget.makedeb.org/debian-feeds/prebuilt-mpr.pub' | gpg --dearmor | sudo tee /usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg 1> /dev/null
    echo "deb [arch=all,$(dpkg --print-architecture) signed-by=/usr/share/keyrings/prebuilt-mpr-archive-keyring.gpg] https://proget.makedeb.org prebuilt-mpr $(lsb_release -cs)" | sudo tee /etc/apt/sources.list.d/prebuilt-mpr.list
    sudo apt update
    sudo apt install tree just
    ```

## Code Structure

The codebase contains the following essential files:

* `Justfile`: the top-level Justfile, entrance to `just` invocations
* `justmods/`:

## Usage

All actions relevant to grading should be made invocable through [`just`](https://github.com/casey/just) recipes. Students need to fill out some of the recipes in the project-level `Justfile`s (e.g., ``) to surface their own KV store system code.

List all `just` recipes:

```bash
just
```

List all files in the codebase as a tree:

```bash
just tree
```

---

Authored by Guanzhou Hu. First offered in CS 739 Spring 2025 taught by Prof. Andrea Arpaci-Dusseau.
