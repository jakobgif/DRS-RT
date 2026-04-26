# DRS-RT Measurement Results and Analysis

This document records the results of the Round-Trip Time (RTT) and jitter measurements performed using the DRS-RT system. It provides an analysis of the observed behavior under different load conditions.

---

## 1. Setup A: Windows PC (Master) to Raspberry Pi 4 (Echo)

**Hardware:**
*   **Master:** Windows PC (via WSL for compilation, running natively on Windows)
*   **Echo:** Raspberry Pi 4 Model B
*   **Network:** Direct Ethernet connection (or via local switch/router)

### 1.1. Scenario T-1: Baseline (Idle)
*   **Condition:** Raspberry Pi is idle.
*   **Cycles:** 50,000
*   **Results:**
    *   **Min RTT:** 339 µs
    *   **Mean RTT:** 609.9 µs
    *   **Max RTT:** 6161 µs
    *   **p99:** 849.0 µs

### 1.2. Scenario T-2: High CPU Load
*   **Condition:** Raspberry Pi CPU saturated with busy-loop workers (`yes > /dev/null` on all 4 cores).
*   **Cycles:** 50,000
*   **Results:**
    *   **Min RTT:** 295 µs
    *   **Mean RTT:** 558.9 µs
    *   **Max RTT:** 6031 µs
    *   **p99:** 799.0 µs

### 1.3. Analysis of Setup A Results

**Observation:**
Counterintuitively, the High CPU Load scenario (T-2) resulted in **lower** minimum, mean, and p99 RTT values compared to the Baseline (T-1). The maximum RTT remained roughly the same (~6 ms).

**Reasoning:**
This phenomenon is a classic example of **CPU Frequency Scaling and Power Management** interactions in general-purpose operating systems like Linux on ARM.

1.  **Baseline (Power Saving):** When the Raspberry Pi is idle (T-1), the Linux `cpufreq` governor scales down the CPU clock speed (e.g., to 600 MHz) to conserve power and reduce thermals. When a network packet arrives, the interrupt wakes the CPU, but there is a latency cost associated with transitioning from a low-power state back to a high-performance state to process the packet and run the Echo logic.
2.  **Load Test (Full Performance):** In Scenario T-2, the background `cpu_load.sh` script pegs all four cores at 100% utilization. This forces the `cpufreq` governor to keep the CPU locked at its maximum clock speed (e.g., 1.5 GHz or 1.8 GHz) constantly.
3.  **The Result:** Because the CPU is already "awake" and running at maximum frequency during the load test, the hardware processes the incoming network interrupts and executes the Echo node reflection code significantly faster than in the baseline scenario. The latency penalty of waking up from an idle state is completely eliminated.

**Conclusion:**
The Linux scheduler on the quad-core Pi 4 is efficient enough to preempt the low-priority user-space `yes` processes almost instantly to handle the incoming UDP packets. To observe true scheduling jitter (where load *increases* RTT), one must force core contention (e.g., by pinning both the load and the Echo node to the same specific core using `taskset`), which prevents the scheduler from utilizing idle resources.

---

## 2. Setup B: Raspberry Pi 4 (Master) to Raspberry Pi 4 (Echo)

*(Placeholder: This section will be populated once measurements between two dedicated Raspberry Pi hardware nodes are completed.)*

**Hardware:**
*   **Master:** Raspberry Pi 4 Model B
*   **Echo:** Raspberry Pi 4 Model B
*   **Network:** Direct Ethernet cable or dedicated real-time switch.

### 2.1. Scenario T-1: Baseline (Idle)
*   **Condition:** Both nodes idle.
*   **Cycles:** 50,000
*   **Results:**
    *   **Min RTT:** [To be determined]
    *   **Mean RTT:** [To be determined]
    *   **Max RTT:** [To be determined]
    *   **p99:** [To be determined]

### 2.2. Scenario T-2: High CPU Load
*   **Condition:** Echo node CPU saturated.
*   **Cycles:** 50,000
*   **Results:**
    *   **Min RTT:** [To be determined]
    *   **Mean RTT:** [To be determined]
    *   **Max RTT:** [To be determined]
    *   **p99:** [To be determined]

### 2.3. Analysis of Setup B Results
*(To be written based on the results.)*
