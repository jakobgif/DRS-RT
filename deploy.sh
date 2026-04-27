#!/bin/bash
# deploy.sh - WSL native deployment script
set -e
PI_ADDR="user@192.168.100.100"

echo "[1/4] Preparing for cross-compilation..."
rustup target add aarch64-unknown-linux-gnu
sudo apt update && sudo apt install gcc-aarch64-linux-gnu -y

echo "[2/4] Building release binary for Raspberry Pi 4 (aarch64)..."
# We don't need to source the env here because we are already in the shell
cargo build --release --target aarch64-unknown-linux-gnu

echo "[3/4] Transferring files to Pi..."
echo "(Enter password 'user' when prompted)"
scp target/aarch64-unknown-linux-gnu/release/drs-rt $PI_ADDR:/home/user/
scp cpu_load.sh $PI_ADDR:/home/user/

echo "[4/4] Setting permissions on Pi..."
ssh $PI_ADDR "chmod +x /home/user/drs-rt /home/user/cpu_load.sh"

echo ""
echo "=== Deployment Complete ==="
echo "To start Echo on Pi:   ./drs-rt echo"
echo "To start Master on Pi: ./drs-rt master --host <ECHO_IP>"
