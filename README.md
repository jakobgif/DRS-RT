# Distributed Real-Time System (DRS) Round-Trip Measurement

UDP-based round-trip time (RTT) and latency jitter measurement between two hardware nodes (e.g., Raspberry Pis), written in Rust.

## System Objectives

- **Master/Echo architecture:** The Master sends a UDP packet; the Echo node reflects it back immediately.
- **RTT recording:** The Master timestamps each send/receive pair with a high-resolution, monotonic clock.
- **Statistics:** Compute minimum, mean, and maximum round-trip times across all cycles.
- **CSV output:** Raw RTT samples are written to a CSV file after the measurement loop for histogram generation.

## DRS Engineering Principles

### The Probe Effect
Logging or writing to disk inside the measurement loop corrupts timing. All samples are buffered in memory during the loop and flushed to disk only after all cycles complete.

### Latency Jitter & WCET
The goal is to capture the full RTT distribution, including rare worst-case tail latencies. Histograms use a logarithmic Y-axis so the WCET tail is never obscured or truncated.

### Error Handling & Omission Failures
UDP does not guarantee delivery. The Master implements a receive timeout to detect lost packets, logs the event, and continues to the next cycle without blocking.

## Test Scenarios

| Scenario | Cycles | Load Condition |
|----------|--------|----------------|
| T-1 | 50,000 | Normal operation (baseline) |
| T-2 | 50,000 | High CPU load (background busy-loop) |
| T-3 | 50,000 | High network load (second binary instance on a different port) |
| T-4 | 500,000 | Normal operation (long-term, captures rare tail latencies) |
