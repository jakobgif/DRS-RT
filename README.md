[![Build Report PDF](https://github.com/jakobgif/DRS-RT/actions/workflows/build-report.yml/badge.svg)](https://github.com/jakobgif/DRS-RT/actions/workflows/build-report.yml)

[![Test](https://github.com/jakobgif/DRS-RT/actions/workflows/test.yml/badge.svg)](https://github.com/jakobgif/DRS-RT/actions/workflows/test.yml)

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

## Usage

### Building

```bash
cargo build --release
```

The binary is placed at `target/release/drs-rt`.

### Echo node

Start the Echo node first. It listens on the given port and reflects every received packet back to the sender unchanged. It runs until terminated with Ctrl+C.

```
drs-rt echo [OPTIONS]

Options:
  --port <PORT>    UDP port to listen on [default: 5000]
  --log  <FILE>    Log file path [default: drs_echo.log]
```

Example:

```bash
./drs-rt echo --port 5000 --log echo.log
```

### Master node

The Master sends UDP packets to the Echo node, records the round-trip time of each cycle, and writes the results to a CSV file after all cycles complete.

```
drs-rt master --host <IP> [OPTIONS]

Required:
  --host <IP>       IP address of the Echo node

Options:
  --port    <PORT>   UDP port of the Echo node [default: 5000]
  --cycles  <N>      Number of measurement cycles [default: 50000]
  --timeout <SECS>   Receive timeout per cycle in seconds [default: 5.0]
  --warmup  <N>      Unrecorded warm-up cycles before measurement [default: 10]
  --cpu-pin <CORE>   Pin the measurement thread to a CPU core (Linux only)
  --output  <FILE>   CSV output file path [default: rtt_results.csv]
  --log     <FILE>   Log file path [default: drs_master.log]
```

Example — run 50,000 cycles against an Echo on `192.168.1.42`:

```bash
./drs-rt master --host 192.168.1.42 --port 5000 --cycles 50000 --output results.csv --log master.log
```

### CSV output format

Each row contains three fields — no header line:

```
<timestamp_us>,<rtt_us>,<status>
```

- `timestamp_us` — microseconds elapsed since the start of the measurement loop
- `rtt_us` — round-trip time in microseconds, or `-1` for lost cycles
- `status` — `ok`, `timeout`, or `seq_mismatch`

### Warm-up cycles

Before the measurement loop the Master performs `--warmup` unrecorded send/receive cycles. These prime the ARP table and CPU caches so that cold-start noise does not appear in the results.

### Lost packets

On a receive timeout the cycle is recorded as `rtt_us=-1, status=timeout` and the loop continues immediately. If the Echo reflects a packet with a mismatched sequence number (stale or reordered), the cycle is recorded as `rtt_us=-1, status=seq_mismatch`.

## Test Scenarios

| Scenario | Cycles | Load Condition |
|----------|--------|----------------|
| T-1 | 50,000 | Normal operation (baseline) |
| T-2 | 50,000 | High CPU load (background busy-loop) |
| T-3 | 50,000 | High network load (second binary instance on a different port) |
| T-4 | 500,000 | Normal operation (long-term, captures rare tail latencies) |
