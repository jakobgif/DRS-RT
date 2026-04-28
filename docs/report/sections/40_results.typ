= Measurement Results

All four test scenarios were executed on the target hardware (Raspberry Pi 4) per the test specification. Each CSV was analysed with `analyze.py`; plots are stored in `measurements_result/`.

== T-1 — Baseline (50 000 cycles)

Normal operation with no external load. Establishes the reference RTT distribution.

#figure(
  table(
    columns: (1fr, auto),
    align: (left, right),
    table.header([*Metric*], [*Value*]),
    [Cycles],   [50 000],
    [Lost],     [0 (0.00 %)],
    [RTT min],  [130 µs],
    [RTT mean], [156.7 µs],
    [RTT max],  [3 004 µs],
    [p99],      [201.0 µs],
    [p99.9],    [426.0 µs],
  ),
  caption: [T-1 Baseline statistics],
)

Under normal conditions the RTT distribution is tight. 99 % of cycles complete within 201 µs. The worst-case sample at 3 004 µs is approximately 19× the mean — a rare scheduling or interrupt event that is visible only on the logarithmic histogram axis. Zero packets were lost.

#figure(
  image("../../../measurements_result/50k.png", width: 100%),
  caption: [T-1 Baseline — RTT distribution and time series],
)

== T-2 — High CPU Load (50 000 cycles)

A background busy-loop saturated all CPU cores during measurement.

#figure(
  table(
    columns: (1fr, auto),
    align: (left, right),
    table.header([*Metric*], [*Value*]),
    [Cycles],   [50 000],
    [Lost],     [0 (0.00 %)],
    [RTT min],  [105 µs],
    [RTT mean], [157.3 µs],
    [RTT max],  [11 729 µs],
    [p99],      [149.0 µs],
    [p99.9],    [8 157.0 µs],
  ),
  caption: [T-2 High CPU load statistics],
)

CPU saturation has negligible impact on the median path (mean 157.3 µs, essentially identical to T-1) but dramatically widens the tail. The p99.9 jumps from 426 µs to 8 157 µs — a 19× increase — and the worst-case sample reaches 11 729 µs, roughly 4× worse than T-1. This confirms that CPU scheduler preemption is the dominant source of worst-case latency jitter on this platform.

#figure(
  image("../../../measurements_result/50k_cpu-load.png", width: 100%),
  caption: [T-2 High CPU load — RTT distribution and time series],
)

== T-3 — High Network Load (50 000 cycles)

A second instance of the binary ran on a different port concurrently, generating competing UDP traffic.

#figure(
  table(
    columns: (1fr, auto),
    align: (left, right),
    table.header([*Metric*], [*Value*]),
    [Cycles],   [50 000],
    [Lost],     [0 (0.00 %)],
    [RTT min],  [108 µs],
    [RTT mean], [143.3 µs],
    [RTT max],  [5 302 µs],
    [p99],      [168.0 µs],
    [p99.9],    [650.0 µs],
  ),
  caption: [T-3 High network load statistics],
)

Competing UDP traffic increases the worst-case sample to 5 302 µs and raises p99.9 to 650 µs, but the effect is notably smaller than CPU load. The mean (143.3 µs) is marginally lower than baseline, likely because the concurrent instance keeps the network stack warm. Network contention causes moderate tail widening without affecting the bulk of the distribution.

#figure(
  image("../../../measurements_result/50k_network-load.png", width: 100%),
  caption: [T-3 High network load — RTT distribution and time series],
)

== T-4 — Long-Term Observation (500 000 cycles)

Normal operation over an extended run to capture rare tail latencies.

#figure(
  table(
    columns: (1fr, auto),
    align: (left, right),
    table.header([*Metric*], [*Value*]),
    [Cycles],   [500 000],
    [Lost],     [0 (0.00 %)],
    [RTT min],  [119 µs],
    [RTT mean], [164.0 µs],
    [RTT max],  [3 424 µs],
    [p99],      [180.0 µs],
    [p99.9],    [221.0 µs],
  ),
  caption: [T-4 Long-term statistics],
)

Across 500 000 cycles the system shows no drift, no packet loss, and a stable mean of 164.0 µs. The tight p99/p99.9 spread (180 / 221 µs) demonstrates that worst-case tail latencies do not grow unboundedly over time. The worst sample at 3 424 µs is comparable to T-1, confirming long-term stability.

#figure(
  image("../../../measurements_result/500k.png", width: 100%),
  caption: [T-4 Long-term — RTT distribution and time series],
)

== Summary

#figure(
  table(
    columns: (auto, auto, auto, auto, auto, auto, auto),
    align: (left, right, right, right, right, right, right),
    table.header(
      [*Scenario*], [*Cycles*], [*Lost*], [*Mean (µs)*], [*p99 (µs)*], [*p99.9 (µs)*], [*Max (µs)*],
    ),
    [T-1 Baseline],      [50 000],  [0], [156.7], [201.0],   [426.0],   [3 004],
    [T-2 CPU load],      [50 000],  [0], [157.3], [149.0],   [8 157.0], [11 729],
    [T-3 Network load],  [50 000],  [0], [143.3], [168.0],   [650.0],   [5 302],
    [T-4 Long-term],     [500 000], [0], [164.0], [180.0],   [221.0],   [3 424],
  ),
  caption: [Comparison across all test scenarios],
)

Key findings:

- *Zero packet loss* across all scenarios and all cycles.
- *Mean RTT is robust* to both CPU and network load (143–164 µs across all scenarios).
- *CPU scheduler preemption* is the dominant jitter source: p99.9 inflates 19× under full CPU saturation compared to baseline.
- *Network contention* causes measurable but moderate tail widening (p99.9 ≈ 1.5× baseline).
- The system is *stable over 500 000 cycles* with no long-term degradation.
