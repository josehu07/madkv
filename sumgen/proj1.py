import argparse
import os
from termcolor import cprint
from pprint import pprint
import re
import matplotlib

matplotlib.use("Agg")

import matplotlib.pyplot as plt


REPORT_MD = """# CS 739 MadKV Project 1

**Group members**: Name `email`, Name `email`

## Design Walkthrough

*FIXME: add your design walkthrough text*

## Self-provided Testcases

<u>Found the following testcase results:</u> {}

### Explanations

*FIXME: add your explanations of each testcase*

## Fuzz Testing

<u>Parsed the following fuzz testing results:</u>

num_clis | conflict | outcome
:-: | :-: | :-:
1 | no | {}
3 | no | {}
3 | yes | {}

### Comments

*FIXME: add your comments on fuzz testing*

## YCSB Benchmarking

<u>Single-client throughput/latency across workloads:</u>

![single-cli]({})

<u>Agg. throughput trend vs. number of clients:</u>

![tput-trend]({})

<u>Avg. & P99 latency trend vs. number of clients:</u>

![lats-trend]({})

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
    for num in range(1, 6):
        head = f"[testcase {num}]"
        cprint(f"  {head:22s}", "cyan", end="")
        print(f"  just p1::testcase {num} <server_addr>")
    for nclis, conflict in [(1, "no"), (3, "no"), (3, "yes")]:
        head = f"[fuzz {nclis} clis {'conflict' if conflict == 'yes' else 'disjoint'}]"
        cprint(f"  {head:22s}", "cyan", end="")
        print(f"  just p1::fuzz {nclis} {conflict} <server_addr>")
    for wload in ["a", "b", "c", "d", "e", "f"]:
        head = f"[ycsb-{wload} 1 clis]"
        cprint(f"  {head:22s}", "cyan", end="")
        print(f"  just p1::bench 1 {wload} <server_addr>")
    for wload in ["a", "c", "e"]:
        for nclis in [10, 25, 40, 55, 70, 85]:
            head = f"[ycsb-{wload} {nclis} clis]"
            cprint(f"  {head:22s}", "cyan", end="")
            print(f"  just p1::bench {nclis} {wload} <server_addr>")

    print(" For each case, launch your server by:  just p1::service 0.0.0.0:<port>")
    print(" on a separate machine, then run a client-side command as listed above;")
    print(" start with a fresh server for each case.")

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


def load_tests_results(result_dir):
    test_logs = os.listdir(f"{result_dir}/tests")

    tests_found = []
    for log in test_logs:
        if log.startswith("test") and log.endswith(".log"):
            tests_found.append(log[4:-4])

    if len(tests_found) == 0:
        raise RuntimeError(f"no testcase results found in '{result_dir}/tests/'")
    return {"testcases": ", ".join(sorted(tests_found))}


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

    for nclis, conflict in [(1, "no"), (3, "no"), (3, "yes")]:
        log = f"{result_dir}/fuzz/fuzz-{nclis}-{conflict}.log"
        check_file_exists(log)
        results[f"fuzz-{nclis}-{conflict}"] = parse_fuzz_result(log)

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

    # single-cli
    wloads_list = ["a", "b", "c", "d", "e", "f"]
    tputs, lat_avgs, lat_p99s = [], [], []
    for wload in wloads_list:
        log = f"{result_dir}/bench/bench-1-{wload}.log"
        check_file_exists(log)
        tput, lats = parse_bench_result(log)
        tputs.append(tput)
        lat_avgs.append(
            sum(lats[op]["ops"] * lats[op]["avg"] for op in lats)
            / sum(lats[op]["ops"] for op in lats)
        )
        lat_p99s.append(max(lats[op]["p99"] for op in lats))

    plt.figure(figsize=(9, 3))
    plt.rc("font", size=12)

    plt.subplot(1, 3, 1)
    plt.bar(wloads_list, tputs)
    plt.xlabel("[Tput]  Workload")
    plt.ylabel("Agg. throughput (ops/sec)")

    plt.subplot(1, 3, 2)
    plt.bar(wloads_list, lat_avgs, hatch="/", edgecolor="lightgray", linewidth=0)
    plt.xlabel("[Avg lat]  Workload")
    plt.ylabel("Avg latency (op-agnostic, us)")

    plt.subplot(1, 3, 3)
    plt.bar(wloads_list, lat_p99s, hatch="x", edgecolor="lightgray", linewidth=0)
    plt.xlabel("[P99 lat]  Workload")
    plt.ylabel("P99 latency (op-agnostic, us)")

    plt.tight_layout()
    plt.savefig(f"{report_dir}/plots/ycsb-single-cli.png", dpi=200)
    plt.close()
    results["ycsb-single-cli"] = "plots/ycsb-single-cli.png"

    # tput-trend
    nclis_list = [1, 10, 25, 40, 55, 70, 85]
    tputs, wlats = dict(), dict()
    for wload in ["a", "c", "e"]:
        tputs[wload] = []
        wlats[wload] = dict()
        for nclis in nclis_list:
            log = f"{result_dir}/bench/bench-{nclis}-{wload}.log"
            check_file_exists(log)
            tput, lats = parse_bench_result(log)
            tputs[wload].append(tput)
            for op in lats:
                if op not in wlats[wload]:
                    wlats[wload][op] = {"avg": [], "p99": []}
                wlats[wload][op]["avg"].append(lats[op]["avg"])
                wlats[wload][op]["p99"].append(lats[op]["p99"])

    plt.figure(figsize=(9, 3))
    plt.rc("font", size=12)

    for i, wload in enumerate(["a", "c", "e"]):
        plt.subplot(1, 3, i + 1)
        plt.plot(nclis_list, tputs[wload], marker="o")
        plt.xlabel(f"[YCSB-{wload}]  #clients")
        plt.ylabel("Agg. throughput (ops/sec)")

    plt.tight_layout()
    plt.savefig(f"{report_dir}/plots/ycsb-tput-trend.png", dpi=200)
    plt.close()
    results["ycsb-tput-trend"] = "plots/ycsb-tput-trend.png"

    # lats-trend
    op_color = {
        "INSERT": "brown",
        "UPDATE": "orange",
        "READ": "royalblue",
        "SCAN": "lightgreen",
    }

    plt.figure(figsize=(9, 3))
    plt.rc("font", size=12)

    for i, wload in enumerate(["a", "c", "e"]):
        plt.subplot(1, 3, i + 1)
        for op in wlats[wload]:
            plt.plot(
                nclis_list,
                wlats[wload][op]["avg"],
                marker="o",
                color=op_color[op],
                linestyle="-",
                label=op,
            )
        plt.xlabel(f"[YCSB-{wload}]  #clients")
        plt.ylabel("Avg latency (us)")
        plt.legend()

    plt.tight_layout()
    plt.savefig(f"{report_dir}/plots/ycsb-lats-trend.png", dpi=200)
    plt.close()
    results["ycsb-lats-trend"] = "plots/ycsb-lats-trend.png"

    return results


def load_all_run_results(result_dir, report_dir):
    tests_results = load_tests_results(result_dir)
    fuzz_results = load_fuzz_results(result_dir)
    bench_results = load_bench_results(result_dir, report_dir)

    return {**tests_results, **fuzz_results, **bench_results}


def generate_report(results, report_dir):
    with open(f"{report_dir}/proj1.md", "w") as f:
        f.write(
            REPORT_MD.format(
                results["testcases"],
                results["fuzz-1-no"],
                results["fuzz-3-no"],
                results["fuzz-3-yes"],
                results["ycsb-single-cli"],
                results["ycsb-tput-trend"],
                results["ycsb-lats-trend"],
            )
        )


if __name__ == "__main__":
    parser = argparse.ArgumentParser()
    parser.add_argument("-i", "--result_dir", type=str, default="/tmp/madkv-p1")
    parser.add_argument("-o", "--report_dir", type=str, default="report")
    args = parser.parse_args()

    expect_configurations()

    check_dir_exists(f"{args.result_dir}/tests")
    check_dir_exists(f"{args.result_dir}/fuzz")
    check_dir_exists(f"{args.result_dir}/bench")
    if not os.path.isdir(f"{args.report_dir}/plots"):
        os.system(f"mkdir -p {args.report_dir}/plots")
    check_dir_exists(f"{args.report_dir}/plots")

    cprint(
        f"Loading logged run results from '{args.result_dir}/'...",
        "yellow",
        attrs=["bold"],
    )
    results = load_all_run_results(args.result_dir, args.report_dir)
    print(f"Done, processed results:")
    pprint(results)

    cprint(
        f"Generating summary report to '{args.report_dir}/proj1.md'...",
        "yellow",
        attrs=["bold"],
    )
    if os.path.isfile(f"{args.report_dir}/proj1.md"):
        cprint(
            "File already exists! Press Enter to overwrite, or Ctrl-C to abort...",
            "blue",
            end="",
        )
        input()
    generate_report(results, args.report_dir)
    print("Generated.")
