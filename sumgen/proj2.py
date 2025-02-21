import argparse
import os
from termcolor import cprint
from pprint import pprint
import re
import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt


REPORT_MD = """# CS 739 MadKV Project 2

**Group members**: Name `email`, Name `email`

## Design Walkthrough

*FIXME: add your design walkthrough text*

## Self-provided Testcase

You will run the described testcase during demo time.

### Explanations

*FIXME: add your explanation of this testcase*

## Fuzz Testing

<u>Parsed the following fuzz testing results:</u>

num_servers | crashing | outcome
:-: | :-: | :-:
3 | no | {}
3 | yes | {}
5 | yes | {}

You will run a crashing/recovering fuzz test during demo time.

### Comments

*FIXME: add your comments on fuzz testing*

## YCSB Benchmarking

<u>10 clients throughput/latency across workloads & number of partitions:</u>

![ten-clients]({})

<u>Agg. throughput trend vs. number of clients w/ and w/o partitioning:</u>

![tput-trend]({})

### Comments

*FIXME: add your discussions of benchmarking results*

## Additional Discussion

*OPTIONAL: add extra discussions if applicable*

"""

ANSI_ESCAPE_RE = re.compile(r"\x1B(?:[@-Z\\-_]|\[[0-?]*[ -/]*[@-~])")


def expect_configurations():
    cprint(
        "Expected tests & benchmarks to run before report generation:",
        "yellow",
        attrs=["bold"],
    )
    for nservers, crashing in [(3, "no"), (3, "yes"), (5, "yes")]:
        head = f"[fuzz {nservers} servers {'crashing' if crashing == 'yes' else 'healthy'}]"
        cprint(f"  {head:25s}", "cyan", end="")
        print(f"  just p2::fuzz {nservers} {crashing} <manager_addr>")
    for wload in ["a", "b", "c", "d", "e", "f"]:
        for nservers in [1, 3, 5]:
            head = f"[ycsb-{wload} {nservers} parts 10 clis]"
            cprint(f"  {head:25s}", "cyan", end="")
            print(f"  just p2::bench 10 {wload} {nservers} <server_addr>")
    for nservers in [1, 5]:
        for nclis in [1, 20, 30]:
            head = f"[ycsb-a {nservers} parts {nclis} clis]"
            cprint(f"  {head:25s}", "cyan", end="")
            print(f"  just p2::bench {nclis} a {nservers} <server_addr>")

    print(" For each case, launch your service by doing:")
    print("    just p2::service <node_id> <manager> <servers> <backer_prefix>")
    print(" for all node IDs, then run a client-side command as listed above;")
    print(" start with fresh servers and storage for each case.")

    cprint(
        "Are all the test and benchmark results ready? Press Enter to continue...",
        "blue",
        end="",
    )
    input()


def check_dir_exists(path):
    if not os.path.isdir(path):
        raise RuntimeError(f"directory '{path}' does not exist")


def check_file_exists(path):
    if not os.path.isfile(path):
        raise RuntimeError(f"log file '{path}' does not exist")


def parse_fuzz_result(fuzz_log):
    with open(fuzz_log, "r") as flog:
        for line in flog:
            line = line.strip()
            if "Fuzz testing result:" in line:
                outcome = ANSI_ESCAPE_RE.sub("", line.split()[-1])
                return outcome

        raise RuntimeError(f"cannot find fuzzing outcome in '{fuzz_log}'")


def load_fuzz_results(result_dir):
    results = dict()

    for nservers, crashing in [(3, "no"), (3, "yes"), (5, "yes")]:
        log = f"{result_dir}/fuzz/fuzz-{nservers}-{crashing}.log"
        check_file_exists(log)
        results[f"fuzz-{nservers}-{crashing}"] = parse_fuzz_result(log)

    return results


def parse_bench_result(ycsb_log):
    tput, lats = 0.0, dict()
    with open(ycsb_log, "r") as flog:
        in_numbers, in_run_sec = False, False
        for line in flog:
            line = line.strip()
            if (not in_numbers) and "Benchmarking results:" in line:
                in_numbers = True
            elif in_numbers and (not in_run_sec) and "[Run]" in line:
                in_run_sec = True
            elif in_run_sec:
                if "Throughput" in line:
                    tput = float(line.split()[-2])
                elif line.endswith(" us"):
                    segs = line.split()
                    op = segs[-12]
                    lats[op] = {
                        "ops": float(segs[-10]),
                        "avg": float(segs[-8]),
                        "p99": float(segs[-2]),
                    }

    if tput is None or len(lats) == 0:
        raise RuntimeError(f"cannot find expected stats in '{ycsb_log}'")
    return tput, lats


def load_bench_results(result_dir, report_dir):
    results = dict()

    # 10 clients
    nservers_list = [1, 3, 5]
    wloads_list = ["a", "b", "c", "d", "e", "f"]
    tputs, lat_avgs, lat_p99s = dict(), dict(), dict()
    for nservers in nservers_list:
        tputs[nservers] = []
        lat_avgs[nservers] = []
        lat_p99s[nservers] = []
        for wload in wloads_list:
            log = f"{result_dir}/bench/bench-10-{wload}-{nservers}.log"
            check_file_exists(log)
            tput, lats = parse_bench_result(log)
            tputs[nservers].append(tput)
            lat_avgs[nservers].append(
                sum(lats[op]["ops"] * lats[op]["avg"] for op in lats)
                / sum(lats[op]["ops"] for op in lats)
            )
            lat_p99s[nservers].append(max(lats[op]["p99"] for op in lats))

    plt.figure(figsize=(10, 3))
    plt.rc("font", size=12)

    plt.subplot(1, 3, 1)
    for off, nservers in enumerate(nservers_list):
        plt.bar(
            [off + i * (len(nservers_list) + 1) for i in range(len(wloads_list))],
            tputs[nservers],
            label=f"{nservers} parts",
        )
    plt.vlines(
        [
            len(nservers_list) + i * (len(nservers_list) + 1)
            for i in range(len(wloads_list))
        ],
        0,
        max(max(l) for l in tputs.values()),
        color="lightgray",
        linestyles=":",
    )
    plt.xticks(
        [1 + i * (len(nservers_list) + 1) for i in range(len(wloads_list))],
        wloads_list,
    )
    plt.xlabel("[Tput]  Workload")
    plt.ylabel("Agg. throughput (ops/sec)")
    plt.legend(loc="lower left", fontsize=10)

    plt.subplot(1, 3, 2)
    for off, nservers in enumerate(nservers_list):
        plt.bar(
            [off + i * (len(nservers_list) + 1) for i in range(len(wloads_list))],
            lat_avgs[nservers],
            hatch="/",
            edgecolor="lightgray",
            linewidth=0,
            label=f"{nservers} parts",
        )
    plt.vlines(
        [
            len(nservers_list) + i * (len(nservers_list) + 1)
            for i in range(len(wloads_list))
        ],
        0,
        max(max(l) for l in lat_avgs.values()),
        color="lightgray",
        linestyles=":",
    )
    plt.xticks(
        [1 + i * (len(nservers_list) + 1) for i in range(len(wloads_list))],
        wloads_list,
    )
    plt.xlabel("[Avg lat]  Workload")
    plt.ylabel("Avg latency (op-agnostic, us)")

    plt.subplot(1, 3, 3)
    for off, nservers in enumerate(nservers_list):
        plt.bar(
            [off + i * (len(nservers_list) + 1) for i in range(len(wloads_list))],
            lat_p99s[nservers],
            hatch="x",
            edgecolor="lightgray",
            linewidth=0,
            label=f"{nservers} parts",
        )
    plt.vlines(
        [
            len(nservers_list) + i * (len(nservers_list) + 1)
            for i in range(len(wloads_list))
        ],
        0,
        max(max(l) for l in lat_p99s.values()),
        color="lightgray",
        linestyles=":",
    )
    plt.xticks(
        [1 + i * (len(nservers_list) + 1) for i in range(len(wloads_list))],
        wloads_list,
    )
    plt.xlabel("[P99 lat]  Workload")
    plt.ylabel("P99 latency (op-agnostic, us)")

    plt.tight_layout()
    plt.savefig(f"{report_dir}/plots-p2/ycsb-ten-clients.png", dpi=200)
    plt.close()
    results["ycsb-ten-clients"] = "plots-p2/ycsb-ten-clients.png"

    # tput-trend
    nservers_list = [1, 5]
    nclis_list = [1, 10, 20, 30]
    tputs = dict()
    for nservers in nservers_list:
        tputs[nservers] = []
        for nclis in nclis_list:
            log = f"{result_dir}/bench/bench-{nclis}-a-{nservers}.log"
            check_file_exists(log)
            tput, _ = parse_bench_result(log)
            tputs[nservers].append(tput)

    plt.figure(figsize=(4, 3))
    plt.rc("font", size=12)

    for nservers in nservers_list:
        plt.plot(
            nclis_list,
            tputs[nservers],
            marker="o",
            label=f"{nservers} parts",
        )
    plt.xlabel(f"[YCSB-a]  #clients")
    plt.ylabel("Agg. throughput (ops/sec)")
    plt.legend(fontsize=10)

    plt.tight_layout()
    plt.savefig(f"{report_dir}/plots-p2/ycsb-tput-trend.png", dpi=200)
    plt.close()
    results["ycsb-tput-trend"] = "plots-p2/ycsb-tput-trend.png"

    return results


def load_all_run_results(result_dir, report_dir):
    fuzz_results = load_fuzz_results(result_dir)
    bench_results = load_bench_results(result_dir, report_dir)

    return {**fuzz_results, **bench_results}


def generate_report(results, report_dir):
    with open(f"{report_dir}/proj2.md", "w") as f:
        f.write(
            REPORT_MD.format(
                results["fuzz-3-no"],
                results["fuzz-3-yes"],
                results["fuzz-5-yes"],
                results["ycsb-ten-clients"],
                results["ycsb-tput-trend"],
            )
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-i", "--result_dir", type=str, default="/tmp/madkv-p2")
    parser.add_argument("-o", "--report_dir", type=str, default="report")
    args = parser.parse_args()

    expect_configurations()

    check_dir_exists(f"{args.result_dir}/fuzz")
    check_dir_exists(f"{args.result_dir}/bench")
    if not os.path.isdir(f"{args.report_dir}/plots-p2"):
        os.system(f"mkdir -p {args.report_dir}/plots-p2")
    check_dir_exists(f"{args.report_dir}/plots-p2")

    cprint(
        f"Loading logged run results from '{args.result_dir}/'...",
        "yellow",
        attrs=["bold"],
    )
    results = load_all_run_results(args.result_dir, args.report_dir)
    print(f"Done, processed results:")
    pprint(results)

    cprint(
        f"Generating summary report to '{args.report_dir}/proj2.md'...",
        "yellow",
        attrs=["bold"],
    )
    if os.path.isfile(f"{args.report_dir}/proj2.md"):
        cprint(
            "File already exists! Press Enter to overwrite, or Ctrl-C to abort...",
            "blue",
            end="",
        )
        input()
    generate_report(results, args.report_dir)
    print("Generated.")
