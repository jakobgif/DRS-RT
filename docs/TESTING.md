# Testing: DRS Round-Trip Measurement

Verification strategy for the Rust binary, Python analysis script, and test tooling. Tests are organized into three levels of increasing fidelity.

---

## Unit Tests

Pure logic, no network or file I/O. Run on any development machine via `cargo test`.

### UT-1 — CLI argument parsing
All valid combinations of role, port, host, cycle count, warmup count, and CPU pin shall parse without error. Invalid combinations (e.g., `--host` on echo role, missing required arguments) shall produce a clear error and a non-zero exit code. Covers F-2, F-3, F-4, F-5, F-18, F-19.

### UT-2 — Buffer pre-allocation
The RTT sample buffer shall be allocated with the full configured capacity before the first send. The test shall verify capacity equals the configured cycle count and that no reallocation occurs during simulated sample insertion. Covers NF-4.

### UT-3 — CSV serialization
The CSV writer shall produce one row per cycle with no header line. Each row shall contain three fields: `timestamp_us`, `rtt_us`, and `status`. The test shall verify row count, column count, value round-trip fidelity, correct line endings, and that each of the three status values (`ok`, `timeout`, `seq_mismatch`) serializes correctly. Covers F-14, F-17.

### UT-4 — Timeout configuration
The receive timeout applied to the socket shall equal the configured timeout value. The test shall verify this without opening a real socket. Covers F-11.

### UT-5 — Packet loss recording
On a simulated timeout, the RTT buffer shall receive a `-1` entry with status `timeout`. The test shall verify the buffer contains exactly one entry, the RTT value is `-1`, and the status is `timeout`. Covers F-12, F-17.

### UT-6 — Sequence number verification
When the Master receives a reply whose sequence number does not match the most recently sent packet, the reply shall be discarded and the cycle treated as lost. The test shall simulate a mismatched reply and verify that the RTT buffer receives a `-1` entry with status `seq_mismatch`. Covers F-15, F-16, F-17.

### UT-7 — Warm-up sample exclusion
The warm-up logic shall not add any sample to the RTT buffer. The test shall simulate W warm-up cycles and verify the RTT buffer remains empty afterwards. Covers F-18.

### UT-8 — CPU affinity binding
When `--cpu-pin <core>` is specified, the affinity-setting function shall bind the calling thread to the given core without error. The test shall call the function directly and verify it returns success. Covers F-19.

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
After all cycles complete, the CSV file shall exist and contain exactly N rows — one per cycle. Successful cycles shall have a positive RTT value and status `ok`. Timeout cycles shall have `-1` and status `timeout`. Discarded cycles (sequence mismatch) shall have `-1` and status `seq_mismatch`. Covers F-12, F-14, F-17.

### IT-6 — No I/O inside the loop
The measurement loop shall not open, write to, or flush any file or stream between the first send and the last receive. Verified structurally: all file handles are opened before and closed after the loop, with no I/O calls inside it. Covers F-10, NF-5.

### IT-7 — Warm-up cycle exclusion
When the Master is run with `--warmup W` and `--cycles N` against an Echo on localhost, the RTT buffer shall contain exactly N entries after the loop completes — not N+W. Covers F-18.

### IT-8 — CPU affinity end-to-end
When the Master is run with `--cpu-pin 0` on Linux, the process shall be bound to core 0 for the duration of the measurement loop. Verified by reading `/proc/<pid>/status` (Cpus_allowed field) from a concurrent observer thread during the run. This test is Linux-only and is skipped on other platforms. Covers F-19.

### IT-9 — Log file creation and content
After a normal run completes, the log file shall exist and be non-empty. After a run with at least one timeout (no Echo present), the log file shall contain an entry for the lost packet. Covers F-13.

---

## System Tests

Run the compiled binary end-to-end on the target hardware (Raspberry Pi). Executed manually following the procedure in @DEVELOPMENT.md. The CI artifact is the authoritative binary for all system tests.

### T-1 — Baseline (50,000 cycles)
Normal operation. Establishes the baseline RTT distribution.

**Pass criteria:** 50,000 rows in CSV, no crashes, log file written.

### T-2 — High CPU load (50,000 cycles)
The CPU load script (TF-1) runs in the background during measurement. Used to observe scheduling jitter.

**Pass criteria:** 50,000 rows in CSV, histogram shows wider distribution than T-1.

### T-3 — High network load (50,000 cycles)
A second instance of the binary runs on a different port concurrently (TF-2). Used to observe network contention jitter.

**Pass criteria:** 50,000 rows in CSV, histogram shows wider distribution than T-1.

### T-4 — Long-term observation (500,000 cycles)
Normal operation over an extended run. Used to capture rare worst-case tail latencies.

**Pass criteria:** 500,000 rows in CSV, no crashes, worst-case tail visible in histogram.
