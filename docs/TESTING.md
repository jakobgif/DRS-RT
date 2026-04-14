# Testing: DRS Round-Trip Measurement

Verification strategy for the Rust binary, Python analysis script, and test tooling. Tests are organized into three levels of increasing fidelity.

---

## Unit Tests

Pure logic, no network or file I/O. Run on any development machine via `cargo test`.

### UT-1 — CLI argument parsing
All valid combinations of role, port, host, and cycle count shall parse without error. Invalid combinations (e.g., `--host` on echo role, missing required arguments) shall produce a clear error and a non-zero exit code. Covers F-2, F-3, F-4, F-5.

### UT-2 — Buffer pre-allocation
The RTT sample buffer shall be allocated with the full configured capacity before the first send. The test shall verify capacity equals the configured cycle count and that no reallocation occurs during simulated sample insertion. Covers NF-4.

### UT-3 — CSV serialization
The CSV writer shall produce one RTT value per row with no header line. The test shall verify row count, value round-trip fidelity, and correct line endings. Covers F-14.

### UT-4 — Timeout configuration
The receive timeout applied to the socket shall equal the configured timeout value. The test shall verify this without opening a real socket. Covers F-11.

### UT-5 — Packet loss counter
On a simulated timeout, the lost-packet counter shall increment by one and the RTT buffer shall not receive an entry. The test shall verify the counter and buffer size independently. Covers F-12.

---

## Integration Tests

Both roles run on `localhost` within the same test process or as child processes. No physical network or Raspberry Pi is required. Run via `cargo test`.

### IT-1 — Echo reflection
A packet sent to the Echo role on localhost shall be returned to the sender byte-for-byte unchanged. Covers F-7.

### IT-2 — RTT recorded positive
After one successful Master–Echo cycle on localhost, the recorded RTT shall be greater than zero. Covers F-6, F-9.

### IT-3 — Full cycle count
After N cycles with an Echo running on localhost, the RTT buffer shall contain exactly N entries. Covers F-5, F-10, NF-4.

### IT-4 — Timeout and continuation
When no Echo is present, the Master shall record a timeout for that cycle and proceed to the next cycle without hanging or crashing. Covers F-11, F-12.

### IT-5 — Post-loop CSV output
After all cycles complete, the CSV file shall exist and contain exactly as many rows as there are successful (non-lost) samples. Covers F-14.

### IT-6 — No I/O inside the loop
The measurement loop shall not open, write to, or flush any file or stream between the first send and the last receive. Verified structurally: all file handles are opened before and closed after the loop, with no I/O calls inside it. Covers F-10, NF-5.

---

## System Tests

Run the compiled binary end-to-end on the target hardware (Raspberry Pi). Executed manually following the procedure in @DEVELOPMENT.md. The CI artifact is the authoritative binary for all system tests.

### T-1 — Baseline (50,000 cycles)
Normal operation. Establishes the baseline RTT distribution.

**Pass criteria:** 50,000 rows in CSV, no crashes, log file written.

### T-2 — High CPU load (50,000 cycles)
The CPU load script (F-18) runs in the background during measurement. Used to observe scheduling jitter.

**Pass criteria:** 50,000 rows in CSV, histogram shows wider distribution than T-1.

### T-3 — High network load (50,000 cycles)
A second instance of the binary runs on a different port concurrently (F-19). Used to observe network contention jitter.

**Pass criteria:** 50,000 rows in CSV, histogram shows wider distribution than T-1.

### T-4 — Long-term observation (500,000 cycles)
Normal operation over an extended run. Used to capture rare worst-case tail latencies.

**Pass criteria:** 500,000 rows in CSV, no crashes, worst-case tail visible in histogram.
