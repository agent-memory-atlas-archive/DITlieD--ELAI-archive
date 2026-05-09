@echo off
REM ELAI Sidecar launcher — opens a visible cmd window with the Python inference server.
REM Invoked by elai.exe --dashboard via `cmd /c start` so the window survives parent exit.
REM Usage: start-sidecar.bat [port]  (defaults to 8100)

set PORT=%1
if "%PORT%"=="" set PORT=8100

@REM Reap any stale sidecar from a prior crash before claiming GPU memory.
@REM This prevents the new instance failing to load (zombie holds ~1.9 GB VRAM).
wsl -d Ubuntu -- pkill -9 -f elai_inference.server 2>nul


title ELAI Sidecar (port %PORT%)
echo === ELAI Inference Sidecar ===
echo Model: Qwen2.5-Coder-3B-Instruct-abliterated (NF4 + TurboQuant 8-bit KV)
echo Port:  %PORT%
echo Loading model weights from disk (takes 30-90s on first launch)...
echo.

wsl -d Ubuntu -- bash -c "source ~/vllm-env/bin/activate && cd /mnt/d/.ELAI_workflow/.elai_cc && PYTHONPATH=python python -m elai_inference.server --model /mnt/d/.ELAI_workflow/.elai_cc/models/Qwen2.5-Coder-3B-Instruct-abliterated --port %PORT% --no-dola --no-steering --turboquant-bits 8 --log-level INFO"

echo.
echo === SIDECAR EXITED (code %ERRORLEVEL%) ===
echo Press any key to close this window.
pause >nul
