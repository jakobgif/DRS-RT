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

### 1.2. Scenario T-2: CPU Load (Simple)
*   **Condition:** Raspberry Pi CPU saturated with `yes > /dev/null`.
*   **Cycles:** 50,000
*   **Results:**
    *   **Min RTT:** 295 µs
    *   **Mean RTT:** 558.9 µs
    *   **Max RTT:** 6031 µs
    *   **p99:** 799.0 µs

### 1.3. Scenario T-2: CPU Load (Aggressive Contention)
*   **Condition:** Raspberry Pi running `aggressive_load.sh` (`stress-ng` targeting Cache, RAM, and Matrix math).
*   **Cycles:** 50,000
*   **Results:**
    *   **Min RTT:** 154 µs
    *   **Mean RTT:** 291.3 µs
    *   **Max RTT:** 8130 µs
    *   **p99:** 553.0 µs

### 1.4. Analysis of Setup A Results

**The "Hot CPU" Paradox:**
In both load scenarios, the **average** performance improved significantly over the idle baseline. This is because the background load prevents the Pi from entering power-saving idle states. At 100% load, the CPU clock is locked at its maximum (1.5GHz+), eliminating the "wake-up" latency for network interrupts.

**Simple vs. Aggressive Load:**
1.  **Simple Load (`yes`):** This is a "clean" load. It uses almost no memory or cache. The CPU can pause it instantly to handle a packet with zero friction.
2.  **Aggressive Load (`stress-ng`):** This load creates **Contention**. It thrashes the L1/L2 caches and saturates the memory bus. 
    *   **The Result:** Even though the average is fast, the **Maximum RTT** spiked to **8.1ms**. 
    *   **Reasoning:** When a network packet arrives, the CPU handles the interrupt, but the Echo node's code and data have been "evicted" from the cache by the stress script. The CPU must wait for slow RAM access to reload the Echo node's memory, leading to the high-latency spikes seen in the "tail" of the histogram.

**Conclusion:**
For real-time systems, "High CPU Usage" is often less dangerous than "High Memory/Cache Contention." Simple math loops might not affect RTT, but heavy data-processing tasks will introduce significant jitter by starving the measurement application of memory bandwidth.

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
