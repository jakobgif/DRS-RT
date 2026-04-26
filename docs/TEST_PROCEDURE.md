# DRS-RT: Manual Test Procedure

This guide provides step-by-step instructions for performing RTT measurements between a Windows PC (Master) and a Raspberry Pi 4 (Echo).

## 1. Build the Binaries

You need to build two versions of the binary: one for your Windows/WSL machine and one for the ARM64 Raspberry Pi.

### Build for Windows (WSL)
In your **WSL Ubuntu terminal**, run:
```bash
cd /mnt/c/Users/bernh/FH/Master/4Semester/DES/DRS-RT
source $HOME/.cargo/env
cargo build --release
```

### Build for Raspberry Pi (ARM64)
In the same **WSL terminal**, run:
```bash
cargo build --release --target aarch64-unknown-linux-gnu
```

---

## 2. Deploy to Raspberry Pi

Copy the Pi-specific binary and the load script to the Pi using **Windows PowerShell**:

```powershell
cd C:\Users\bernh\FH\Master\4Semester\DES\DRS-RT

# Copy binary
scp target\aarch64-unknown-linux-gnu\release\drs-rt user@192.168.100.100:/home/user/

# Copy load script
scp cpu_load.sh user@192.168.100.100:/home/user/

# Set permissions
ssh user@192.168.100.100 "chmod +x /home/user/drs-rt /home/user/cpu_load.sh"
```

---

## 3. Execute Scenario T-1: Baseline (Idle Pi)

### Step A: Prepare the Pi
In your **SSH window** to the Pi:
1. Ensure no load is running: `pkill yes`
2. Start the Echo node: `./drs-rt echo`

### Step B: Run the Measurement
In your **Windows PowerShell**:
```powershell
wsl -e ./target/release/drs-rt master --host 192.168.100.100 --cycles 50000 --output baseline.csv
```

---

## 4. Execute Scenario T-2: High CPU Load (Busy Pi)

### Step A: Prepare the Pi
In your **SSH window** to the Pi:
1. Stop any old echo: `pkill -f drs-rt`
2. Start the CPU load: `./cpu_load.sh &`
3. Verify 100% usage (optional): `htop`
4. Start the Echo node: `./drs-rt echo`

### Step B: Run the Measurement
In your **Windows PowerShell**:
```powershell
wsl -e ./target/release/drs-rt master --host 192.168.100.100 --cycles 50000 --output load_test.csv
```

### Step C: Cleanup the Pi
In your **SSH window**:
```bash
pkill yes
pkill -f drs-rt
```

---

## 5. Analyze Results

In your **Windows PowerShell**, run the analysis for either file:

```powershell
# Analyze baseline
python analyze.py baseline.csv

# Analyze load test
python analyze.py load_test.csv
```

Check the generated `.png` files to compare the jitter tails between the two scenarios.
