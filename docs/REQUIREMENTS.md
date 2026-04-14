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
The Master shall implement a **receive timeout** to detect lost UDP packets.

#### F-12 — Graceful packet loss handling
On timeout (packet loss), the Master shall log the lost cycle and continue to the next cycle without blocking.

#### F-13 — Log file output
All log output (errors, lost packets, status messages) shall be written to a **log file**. Logs shall not be printed to the console during the measurement loop to avoid the probe effect.

#### F-14 — CSV output
After all cycles complete, the Master shall write all samples to a **CSV file** with one RTT value per row.

### Non-Functional Requirements

#### NF-1 — Language
The implementation shall be written in **Rust** using only safe code where possible.

#### NF-2 — Timing accuracy
The timing mechanism shall not be affected by wall-clock adjustments and shall provide sufficient resolution for microsecond-level measurements.

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

#### F-15 — CSV file path via CLI argument
The script shall accept the path to the CSV file as a command-line argument (e.g., `python analyze.py results.csv`).

#### F-16 — RTT statistics
The script shall compute the **minimum**, **mean**, and **maximum** RTT from the CSV data.

#### F-17 — Histogram visualization
The script shall generate histograms from the CSV data. The Y-axis shall use a **logarithmic scale** to reveal the WCET tail. The longest delays shall never be cut off.

### Non-Functional Requirements

#### NF-7 — Full sample preservation
All samples shall be included in the analysis, including outliers and worst-case values — no truncation.

---

## Component 3: Test Tooling

Supporting scripts used to create controlled load conditions for the test scenarios. These are not part of the measurement system and do not need to be precise.

### Functional Requirements

#### F-18 — CPU load script
A bash script shall be provided that saturates all available CPU cores with busy-loop workers. It shall run in the background and be stoppable via Ctrl+C or by killing the process group.

#### F-19 — Network load scenario
The high network load scenario (T-3) shall be achieved by running a second instance of the Rust binary on a different port. No additional script is required.

---

## Test Scenarios

### T-1 — Baseline (50,000 cycles)
Normal operation. Establishes the baseline RTT distribution.

### T-2 — High CPU load (50,000 cycles)
A background stress process generates CPU spikes during measurement. Used to observe scheduling jitter.

### T-3 — High network load (50,000 cycles)
A second instance of the ping-pong program runs on a different port concurrently. Used to observe network contention jitter.

### T-4 — Long-term observation (500,000 cycles)
Normal operation over an extended run. Used to capture rare worst-case tail latencies.

---

## Open Questions

### Q-1 — Packet loss timeout value
What is the timeout duration for declaring a packet lost? (e.g., 100 ms, 1 s)

### Q-2 — Packet payload
What is the packet payload? Options: fixed-size padding, embedded sequence number, embedded timestamp.

### Q-3 — Sequence number verification
Should the Master verify that the echoed packet matches the sent packet via a sequence number check?

### Q-4 — CSV timestamp column
Should the CSV include a wall-clock timestamp per sample in addition to the RTT value?

### Q-5 — Inter-cycle pacing
Is a fixed inter-cycle send interval required, or should the Master send the next packet immediately after receiving a reply (send-and-wait as fast as possible)? Note: a fixed interval would conflict with NF-3 (no artificial delays) and would require an explicit exception.

### Q-6 — Packet loss logging inside the measurement loop
F-12 requires lost packets to be logged, but F-10 and NF-5 prohibit any I/O inside the measurement loop. Should lost-packet events be buffered in memory during the loop and flushed to the log file only after all cycles complete — consistent with how RTT samples are handled?

### Q-7 — Packet payload and sequence verification
Q-2 and Q-3 are coupled: if the payload is fixed-size padding only, sequence number verification (Q-3) is not possible. If a sequence number is embedded in the payload, stale or reordered echo replies can be detected. Which approach is required?
