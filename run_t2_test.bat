@echo off
setlocal
echo [DRS-RT] Automated Scenario T-2 Test (Localhost)

:: 1. Ensure the binary is built
echo [1/6] Building project in WSL...
wsl bash -c "source $HOME/.cargo/env && cargo build --release"

:: 2. Start the Echo node in the background
echo [2/6] Starting Echo node...
start /b wsl bash -c "source $HOME/.cargo/env && ./target/release/drs-rt echo"
:: Wait for the socket to bind
timeout /t 2 /nobreak >nul

:: 3. Start the CPU load in the background
echo [3/6] Starting CPU Load...
start /b wsl ./cpu_load.sh
:: Wait for cores to saturate
timeout /t 2 /nobreak >nul

:: 4. Run the Master Measurement
echo [4/6] Running Master Measurement (10,000 cycles)...
wsl bash -c "source $HOME/.cargo/env && ./target/release/drs-rt master --host 127.0.0.1 --cycles 10000 --output t2_test.csv --log t2_test.log"

:: 5. Clean up background processes
echo [5/6] Cleaning up...
wsl pkill -f drs-rt
wsl pkill yes
wsl pkill -f cpu_load.sh

:: 6. Analyze results
echo [6/6] Analyzing results...
python analyze.py t2_test.csv --out t2_test.png

echo.
echo === Test Complete ===
echo Results: t2_test.csv
echo Plot:    t2_test.png
echo Log:     t2_test.log
pause
