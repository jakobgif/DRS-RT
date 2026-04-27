@echo off
setlocal
set PI_IP=192.168.100.100
set PI_USER=user
set PI_ADDR=%PI_USER%@%PI_IP%

if "%~1"=="" goto usage
if /i "%~1"=="baseline" goto baseline
if /i "%~1"=="load" goto load
goto usage

:baseline
echo === SCENARIO T-1: BASELINE ===
echo [1/4] Ensuring Pi is IDLE (stopping any background load)...
ssh %PI_ADDR% "pkill yes > /dev/null 2>&1 || true"
goto common

:load
echo === SCENARIO T-2: HIGH CPU LOAD ===
echo [1/4] Starting busy-loop workers on Pi...
:: Fix line endings and run in background
ssh -n %PI_ADDR% "sed -i 's/\r//' /home/user/cpu_load.sh; nohup /home/user/cpu_load.sh < /dev/null > /home/user/load.log 2>&1 &"
:: Give it a moment to saturate
timeout /t 2 /nobreak >nul
goto common

:common
echo [2/4] Starting Echo node on Pi...
:: Kill old echo. Use absolute paths.
ssh -n %PI_ADDR% "pkill -f drs-rt > /dev/null 2>&1 || true; nohup /home/user/drs-rt echo < /dev/null > /home/user/echo.log 2>&1 &"
timeout /t 4 /nobreak >nul

echo [3/4] Running Master Measurement (50,000 cycles)...
:: Added --timeout 0.5 to prevent long hangs
wsl -e ./target/release/drs-rt master --host %PI_IP% --cycles 50000 --timeout 0.5 --output %~1_results.csv --log %~1.log

echo [4/4] Cleaning up Pi...
ssh %PI_ADDR% "pkill yes; pkill -f drs-rt"

echo Analyzing results...
python analyze.py %~1_results.csv --out %~1_plot.png

echo.
echo === Test Complete ===
echo Data: %~1_results.csv
echo Plot: %~1_plot.png
pause
exit /b

:usage
echo Usage: run_pi_test.bat [baseline ^| load]
echo.
echo   baseline - Stops load workers on Pi, then measures
echo   load     - Starts load workers on Pi, then measures
exit /b 1
