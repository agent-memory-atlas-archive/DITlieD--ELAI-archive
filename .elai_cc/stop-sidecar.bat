@echo off
REM ELAI Sidecar stopper — sends SIGTERM to the WSL2 Python process so the in-process
REM GPU cleanup handler fires, then waits 5 seconds and sends SIGKILL as a backstop.
REM Run from Windows host to cleanly release VRAM without a reboot.
REM
REM Why WSL2 kill, not taskkill?
REM   taskkill targets the wsl.exe Windows stub, not the Linux python process.
REM   The Linux process keeps running in the Hyper-V VM with its CUDA allocations intact.
REM   `wsl -d Ubuntu -- pkill` sends the signal inside the VM's process namespace,
REM   which is the only reliable kill path short of `wsl --terminate Ubuntu`.

echo Killing ELAI sidecar (SIGTERM, then SIGKILL after 5s)...

wsl -d Ubuntu -- pkill -TERM -f elai_inference.server 2>nul
timeout /t 5 /nobreak >nul
wsl -d Ubuntu -- pkill -KILL -f elai_inference.server 2>nul

echo Done.
