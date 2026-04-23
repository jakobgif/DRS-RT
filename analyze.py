"""
RTT analysis and visualization for drs-rt CSV output.

Usage:
    python analyze.py <results.csv> [--out <output.png>]

CSV format (no header): timestamp_us,rtt_us,status
"""

import argparse
import os
import sys

import matplotlib.pyplot as plt
import matplotlib.ticker as ticker
import numpy as np


def load_csv(path):
    timestamps, rtts, statuses = [], [], []
    with open(path) as f:
        for line in f:
            line = line.strip()
            if not line:
                continue
            ts, rtt, status = line.split(",")
            timestamps.append(int(ts))
            rtts.append(int(rtt))
            statuses.append(status)
    return (
        np.array(timestamps, dtype=np.int64),
        np.array(rtts, dtype=np.int64),
        np.array(statuses, dtype=object),
    )


def print_stats(rtts, statuses):
    ok_mask = statuses == "ok"
    ok_rtts = rtts[ok_mask]
    n_total = len(rtts)
    n_ok = ok_mask.sum()
    n_lost = n_total - n_ok

    print(f"Cycles      : {n_total:,}")
    print(f"  ok        : {n_ok:,}")
    print(f"  lost      : {n_lost:,}  ({100 * n_lost / n_total:.2f} %)")
    if n_ok > 0:
        print(f"RTT (us)")
        print(f"  min       : {ok_rtts.min()}")
        print(f"  mean      : {ok_rtts.mean():.1f}")
        print(f"  max       : {ok_rtts.max()}")
        print(f"  p99       : {np.percentile(ok_rtts, 99):.1f}")
        print(f"  p99.9     : {np.percentile(ok_rtts, 99.9):.1f}")


def plot(timestamps, rtts, statuses, out_path, csv_name):
    ok_mask = statuses == "ok"
    to_mask = statuses == "timeout"
    sm_mask = statuses == "seq_mismatch"

    ok_ts = timestamps[ok_mask] / 1e6          # seconds
    ok_rtts = rtts[ok_mask]
    lost_ts = timestamps[~ok_mask] / 1e6

    fig, (ax_hist, ax_time) = plt.subplots(
        2, 1, figsize=(12, 8), constrained_layout=True
    )
    fig.suptitle(f"RTT Analysis — {csv_name}", fontsize=13)

    # ── Histogram (PF-3) ────────────────────────────────────────────────────────
    if ok_rtts.size > 0:
        ax_hist.hist(ok_rtts, bins="auto", color="steelblue", edgecolor="none")
        ax_hist.set_yscale("log")
        ax_hist.set_xlabel("RTT (µs)")
        ax_hist.set_ylabel("Count (log scale)")
        ax_hist.set_title("RTT Distribution")
        ax_hist.yaxis.set_major_formatter(ticker.LogFormatter())
        ax_hist.grid(axis="y", linestyle="--", alpha=0.4)

        # annotate stats
        stats_text = (
            f"n={ok_rtts.size:,}  "
            f"min={ok_rtts.min()}  "
            f"mean={ok_rtts.mean():.1f}  "
            f"max={ok_rtts.max()}  "
            f"p99={np.percentile(ok_rtts, 99):.1f} µs"
        )
        ax_hist.set_title(f"RTT Distribution — {stats_text}", fontsize=10)

    # ── RTT over time ───────────────────────────────────────────────────────────
    if ok_rtts.size > 0:
        ax_time.scatter(
            ok_ts, ok_rtts,
            s=0.5, color="steelblue", alpha=0.4, linewidths=0, label="ok",
            rasterized=True,
        )

    # mark lost cycles as red ticks at the bottom
    if to_mask.any():
        ax_time.scatter(
            timestamps[to_mask] / 1e6,
            np.zeros(to_mask.sum()),
            s=6, color="crimson", marker="|", label="timeout", zorder=3,
        )
    if sm_mask.any():
        ax_time.scatter(
            timestamps[sm_mask] / 1e6,
            np.zeros(sm_mask.sum()),
            s=6, color="darkorange", marker="|", label="seq_mismatch", zorder=3,
        )

    ax_time.set_xlabel("Time (s)")
    ax_time.set_ylabel("RTT (µs)")
    ax_time.set_title("RTT over Time")
    ax_time.grid(linestyle="--", alpha=0.3)
    if to_mask.any() or sm_mask.any():
        ax_time.legend(markerscale=4, fontsize=8)

    fig.savefig(out_path, dpi=150)
    print(f"Plot saved to {out_path}")


def main():
    parser = argparse.ArgumentParser(description="Analyze drs-rt RTT CSV output.")
    parser.add_argument("csv", help="Path to the CSV file produced by drs-rt master")
    parser.add_argument(
        "--out",
        default=None,
        help="Output PNG path (default: <csv_basename>.png next to the CSV)",
    )
    args = parser.parse_args()

    if not os.path.isfile(args.csv):
        sys.exit(f"error: file not found: {args.csv}")

    out_path = args.out or os.path.splitext(args.csv)[0] + ".png"
    csv_name = os.path.basename(args.csv)

    timestamps, rtts, statuses = load_csv(args.csv)
    print_stats(rtts, statuses)
    plot(timestamps, rtts, statuses, out_path, csv_name)


if __name__ == "__main__":
    main()
