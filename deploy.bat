@echo off
setlocal
set PI_ADDR=user@192.168.100.100

echo [1/3] Building for Raspberry Pi 4 via WSL...
:: This uses the Rust toolchain you already installed in WSL
wsl bash -c "source $HOME/.cargo/env && cargo build --release --target aarch64-unknown-linux-gnu"

if %ERRORLEVEL% NEQ 0 (
    echo.
    echo [ERROR] Build failed. Please check your WSL Rust installation.
    exit /b %ERRORLEVEL%
)

echo [2/3] Transferring files to Pi at %PI_ADDR%...
echo (Enter password 'user' when prompted)
scp target\aarch64-unknown-linux-gnu\release\drs-rt %PI_ADDR%:/home/user/
scp cpu_load.sh %PI_ADDR%:/home/user/

echo [3/3] Setting permissions on Pi...
ssh %PI_ADDR% "chmod +x /home/user/drs-rt /home/user/cpu_load.sh"

echo.
echo === Deployment Complete ===
echo You can now run the tool on your Pi.
echo To start Echo on Pi:   ./drs-rt echo
echo To start Master on Pi: ./drs-rt master --host <ECHO_IP>
