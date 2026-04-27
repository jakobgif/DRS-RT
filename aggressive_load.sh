#!/bin/bash
# aggressive_load.sh - Scenario T-2/T-3 Aggressive Load
# Targets Cache and Memory bus contention to maximize RTT jitter.

# Check if stress-ng is installed
if ! command -v stress-ng &> /dev/null; then
    echo "stress-ng not found. Installing..."
    sudo apt update && sudo apt install stress-ng -y
fi

echo "Starting Aggressive Contention Load..."
echo "Targeting: CPU Pipeline, L1/L2 Cache, and RAM Bus"
echo "Press Ctrl+C to stop."

# --cache 0: Thrash cache on all cores
# --vm 2 --vm-bytes 128M: Memory contention
# --switch 4: Force frequent context switching
stress-ng --cache 0 --vm 2 --vm-bytes 128M --switch 4 --matrix 0
