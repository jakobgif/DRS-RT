# Requirements: DRS Round-Trip Measurement

**Project:** Distributed Real-Time System — UDP RTT & Jitter Measurement  
**Nodes:** Master + Echo (e.g., two Raspberry Pis)

---

## Component 1: Rust Binary

The Rust binary is the measurement engine. It runs on both nodes — the role is selected at runtime via CLI argument. Performance is the top priority. Every decision must be evaluated for its impact on latency. The measurement loop is the hot path — nothing that adds latency belongs inside it.

### Functional Requirements

#### F-1 — Dual-role architecture
The system shall consist of two roles: a **Master** and an **Echo** node.

#### F-2 — Single binary, runtime role selection
The system shall be delivered as a **single binary**. The node role shall be selected at runtime via a command-line argument (e.g., `./drs-rt master` / `./drs-rt echo`).

#### F-3 — Configurable port with default
The UDP port shall be configurable via an optional CLI argument. If not specified, a default port shall be used (e.g., `./drs-rt master` uses the default; `./drs-rt master --port 9000` overrides it).

#### F-4 — Configurable peer IP address
The Master shall accept the Echo node's IP address as a CLI argument (e.g., `./drs-rt master --host 192.168.1.42`). The Echo node does not require a target address as it replies to the sender.

#### F-5 — Configurable cycle count (Master only)
The number of measurement cycles shall be configurable via a command-line argument. This argument only applies when running in master mode and shall be ignored or rejected when running in echo mode.

#### F-6 — Master sends and awaits reply
The Master shall send a UDP packet to the Echo node and wait for a reply.

#### F-7 — Echo reflects packets
The Echo node shall reflect every received UDP packet back to the Master immediately, without modification.

#### F-8 — Echo runs until terminated
The Echo node shall run indefinitely, reflecting packets until terminated by the user (e.g., Ctrl+C).

#### F-9 — RTT recording
The Master shall record the round-trip time (RTT) for each cycle with sufficient precision to capture microsecond-level variations.

#### F-10 — In-memory buffering (no probe effect)
All RTT samples shall be buffered in memory during the measurement loop. No file I/O or console output shall occur inside the loop. **Data logging must have zero impact on the RTT measurement** — writing to disk or any output stream is strictly deferred until all cycles are complete.

#### F-11 — Receive timeout
The Master shall implement a configurable receive timeout (`--timeout <seconds>`, default **5 s**) to detect lost UDP packets.

#### F-12 — Graceful packet loss handling
On timeout (packet loss), the Master shall record the lost cycle in an in-memory buffer and continue to the next cycle without blocking. Lost-packet events shall be flushed to the log file only after all cycles complete, consistent with F-10 and NF-5.

#### F-13 — Log file output
All log output (errors, lost packets, status messages) shall be written to a **log file**. Logs shall not be printed to the console during the measurement loop to avoid the probe effect.

#### F-14 — CSV output
After all cycles complete, the Master shall write all samples to a **CSV file** with one RTT value per row.

#### F-15 — Sequence number payload
Every UDP packet sent by the Master shall carry a **`u64` sequence number** as its payload. The Echo node reflects the packet back byte-for-byte, preserving the sequence number.

#### F-16 — Stale and reordered packet handling
On receive, the Master shall verify that the sequence number in the reply matches the sequence number of the most recently sent packet. Replies with a mismatched sequence number (stale or reordered) shall be discarded and the cycle shall be treated as lost (same handling as a timeout).

#### F-17 — Relative timestamp in CSV
Each row in the CSV file shall include a **relative timestamp** (microseconds elapsed since the start of the measurement loop) alongside the RTT value. This enables time-series analysis and correlation of RTT spikes with system events.

### Non-Functional Requirements

#### NF-1 — Language
The implementation shall be written in **Rust** using only safe code where possible.

#### NF-2 — Timing accuracy
Timing shall use `std::time::Instant`, which maps to `CLOCK_MONOTONIC` (via vDSO, no syscall) on Linux and `QueryPerformanceCounter` on Windows. Both are monotonic and immune to wall-clock adjustments. On the Raspberry Pi 4, the underlying ARM System Counter runs at 19.2 MHz (~52 ns resolution), which is sufficient for microsecond-level RTT measurements. The `Instant::now()` call shall appear only at send and receive — never inside any other hot-path logic.

#### NF-3 — No artificial delays
The system shall not introduce artificial delays (e.g., `sleep`) between cycles unless explicitly required.

#### NF-4 — Pre-allocated buffer
Memory allocation for the RTT buffer shall be done **before** the measurement loop begins to avoid allocation jitter during measurement.

#### NF-5 — Hot path discipline
No dynamic memory allocation, file I/O, console output, or other high-latency operations shall occur inside the measurement loop. If an operation could add latency, it must be moved outside the loop or eliminated.

#### NF-6 — Socket error resilience
The system shall handle UDP socket errors without crashing; errors shall be logged and the loop shall continue.

---

## Component 2: Python Analysis Script

The Python script is responsible for all post-measurement analysis and visualization. It operates solely on the CSV file produced by the Rust binary.

### Functional Requirements

#### PF-1 — CSV file path via CLI argument
The script shall accept the path to the CSV file as a command-line argument (e.g., `python analyze.py results.csv`).

#### PF-2 — RTT statistics
The script shall compute the **minimum**, **mean**, and **maximum** RTT from the CSV data.

#### PF-3 — Histogram visualization
The script shall generate histograms from the CSV data. The Y-axis shall use a **logarithmic scale** to reveal the WCET tail. The longest delays shall never be cut off.

### Non-Functional Requirements

#### PNF-1 — Full sample preservation
All samples shall be included in the analysis, including outliers and worst-case values — no truncation.

---

## Component 3: Test Tooling

Supporting scripts used to create controlled load conditions for the test scenarios. These are not part of the measurement system and do not need to be precise.

### Functional Requirements

#### TF-1 — CPU load script
A bash script shall be provided that saturates all available CPU cores with busy-loop workers. It shall run in the background and be stoppable via Ctrl+C or by killing the process group.

#### TF-2 — Network load scenario
The high network load scenario (T-3) shall be achieved by running a second instance of the Rust binary on a different port. No additional script is required.

---

## Test Scenarios

See @TESTING.md for the full test specification (unit, integration, and system tests).

### T-1 — Baseline (50,000 cycles)
Normal operation. Establishes the baseline RTT distribution.

### T-2 — High CPU load (50,000 cycles)
A background stress process generates CPU spikes during measurement. Used to observe scheduling jitter.

### T-3 — High network load (50,000 cycles)
A second instance of the ping-pong program runs on a different port concurrently. Used to observe network contention jitter.

### T-4 — Long-term observation (500,000 cycles)
Normal operation over an extended run. Used to capture rare worst-case tail latencies.

---

# AI ASSISTANT ANALYSIS & PROPOSED UPDATES

## Potential Missing Requirements

1.  **M-1 - Configurable Payload Size:** RTT varies significantly with packet size. A CLI argument (e.g., --size) should be added to control the amount of padding added after the sequence number.
2.  **M-2 - Warm-up Cycles:** To avoid 'cold start' noise (ARP, cache misses), the Master should perform a configurable number of non-recorded warm-up cycles (e.g., default 10).
3.  **M-3 - Signal Handling (Partial Data Preservation):** If the user stops the Master with Ctrl+C, the binary should gracefully exit the loop and write the *currently collected* buffer to the CSV before terminating.
4.  **M-4 - Buffer Overflow Protection:** The binary should check available system memory against the requested cycle count * sample size before allocation to prevent OOM crashes.
5.  **M-5 - CPU Affinity (Real-Time):** For higher precision on the Raspberry Pi, an optional argument to pin the measurement thread to a specific CPU core (e.g., --cpu-pin 3) would reduce scheduling jitter.

## Observations

1.  **Consistency Error (TESTING.md vs REQUIREMENTS.md):** TESTING.md (IT-5) suggests only successful samples are in the CSV, but for jitter analysis, knowing where gaps (lost packets) occurred is vital. I recommend the CSV always has cycle count rows.
2.  **Jitter Definition:** While 'Jitter' is in the project title, no specific formula is requested. I recommend implementing the **Standard Deviation of RTT** and **Peak-to-Peak Jitter** in the Python analysis script.
3.  **Clock Resolution:** While std::time::Instant is monotonic, its precision on various OS/Hardware combinations should be verified during the first run and logged.
