= Project Overview

This lab report documents the design, implementation, and measurement results of a UDP-based round-trip time (RTT) and latency jitter measurement system, built as part of the Distributed Embedded Systems course. The system targets two hardware nodes (Raspberry Pi 4) connected over a local network and is implemented in Rust.

== Motivation

Real-time distributed systems must satisfy strict timing constraints. Understanding the round-trip latency between two nodes, and critically its worst-case behavior, is essential for validating whether a communication link meets those constraints. A single mean RTT value is insufficient; the full distribution, including rare tail latencies, must be captured and analyzed.

== System Architecture

The system follows a *Master / Echo* architecture:

- *Master node:* Sends a UDP packet, waits for a reply, and records the elapsed time as the round-trip time for that cycle. After all cycles complete, it writes the collected samples to a CSV file and a log file.
- *Echo node:* Listens for incoming UDP packets and immediately reflects each one back to the sender, without modification.

Both roles are delivered as a *single Rust binary*. The role is selected at runtime via a command-line argument (`master` or `echo`). All measurement parameters (target IP, port, cycle count, and receive timeout) are configurable via CLI flags.

== Measurement Discipline

To avoid the *probe effect* (where the act of measuring corrupts the measurement), all RTT samples are held in a pre-allocated in-memory buffer during the measurement loop. No file I/O, console output, or dynamic memory allocation occurs inside the loop. Samples are flushed to disk only after the loop has completed.

The Master uses `std::time::Instant` for timestamping, which maps to a monotonic hardware counter (ARM System Counter at 19.2 MHz on the Raspberry Pi 4, giving ~52 ns resolution). Timestamps are recorded immediately before the send and immediately after the receive, with no intervening logic.

== Analysis

A Python script consumes the CSV output produced by the Master. It computes the minimum, mean, and maximum RTT across all samples and generates a histogram with a logarithmic Y-axis so that rare worst-case tail latencies remain visible and are never obscured.

== Test Scenarios

Four test scenarios are defined to characterize the system under different load conditions:

#table(
  columns: (auto, auto, 1fr),
  table.header([*Scenario*], [*Cycles*], [*Load Condition*]),
  [T-1], [50,000],  [Normal operation (baseline)],
  [T-2], [50,000],  [High CPU load (background busy-loop)],
  [T-3], [50,000],  [High network load (second binary instance on a different port)],
  [T-4], [500,000], [Normal operation, long-term (captures rare tail latencies)],
)
