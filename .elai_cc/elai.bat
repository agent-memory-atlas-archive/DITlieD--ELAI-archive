@echo off
REM ELAI Local Mode — runs the full CLI against local Gemma4 E2B with DoLa + TurboQuant.
REM Starts the inference sidecar if not already running, then launches the CLI.

REM Check if sidecar is already running
curl -s http://localhost:8100/health >nul 2>&1
if %ERRORLEVEL% neq 0 (
    echo Starting inference sidecar...
    start /b "" wsl -d Ubuntu -- bash -c "source ~/vllm-env/bin/activate && cd /mnt/d/.ELAI_workflow/.elai_cc && python -m python.elai_inference.server --model google/gemma-4-E2B-it --port 8100 --dola --turboquant-bits 4 --log-level WARNING" >nul 2>&1

    :wait
    curl -s http://localhost:8100/health >nul 2>&1
    if %ERRORLEVEL% neq 0 (
        timeout /t 2 /nobreak >nul
        goto wait
    )
    echo Sidecar ready.
)

REM Point the CLI at the local sidecar instead of Anthropic
set ANTHROPIC_BASE_URL=http://localhost:8100
set ANTHROPIC_API_KEY=local

REM Launch the full CLI
"%~dp0target\release\elai.exe" %*
