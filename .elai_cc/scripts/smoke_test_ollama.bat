@echo off
:: B-15A Task 11 — Ollama smoke test (Windows sibling).
::
:: EXPECTATIONS:
::   - if ollama is NOT running on localhost:11434, exits 0 with "skipping".
::   - if ollama IS running, posts a canary /api/generate and asserts a
::     non-empty `response` field.
::   - exit 0 on skip or success; exit 1 on a positive-but-unhealthy probe.
::   - --help / /? prints usage and exits 0.

setlocal EnableExtensions EnableDelayedExpansion

if "%~1"=="--help" goto :usage
if "%~1"=="-h"     goto :usage
if "%~1"=="/?"     goto :usage

if "%OLLAMA_HOST%"==""  set "OLLAMA_HOST=http://localhost:11434"
if "%OLLAMA_MODEL%"=="" set "OLLAMA_MODEL=qwen2.5-coder:3b"

:: Reachability + probe via PowerShell (Windows always has it; curl is
:: optional on older Win10 SKUs).
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "try { $r = Invoke-WebRequest -UseBasicParsing -Uri ($env:OLLAMA_HOST + '/api/tags') -TimeoutSec 2 -ErrorAction Stop } catch { Write-Output ('smoke_test_ollama: ollama not running at ' + $env:OLLAMA_HOST + '; skipping.'); exit 0 }; ^
     Write-Output ('smoke_test_ollama: ollama reachable at ' + $env:OLLAMA_HOST + '; sending canary.'); ^
     $payload = @{ model = $env:OLLAMA_MODEL; prompt = 'ping'; stream = $false } ^| ConvertTo-Json -Compress; ^
     try { $resp = Invoke-RestMethod -Uri ($env:OLLAMA_HOST + '/api/generate') -Method Post -Body $payload -ContentType 'application/json' -TimeoutSec 30 -ErrorAction Stop } catch { Write-Error ('smoke_test_ollama: POST /api/generate failed: ' + $_.Exception.Message); exit 1 }; ^
     if ([string]::IsNullOrEmpty($resp.response)) { Write-Error 'smoke_test_ollama: response field empty or missing.'; exit 1 } else { Write-Output 'smoke_test_ollama: PASS (response field non-empty).'; exit 0 }"

exit /b %ERRORLEVEL%

:usage
echo Usage: smoke_test_ollama.bat [--help^|-h^|/?]
echo.
echo Pings ollama; if offline, skips with exit 0. If online, sends a canary
echo /api/generate and asserts non-empty response field.
echo.
echo Environment:
echo   OLLAMA_HOST   default http://localhost:11434
echo   OLLAMA_MODEL  default qwen2.5-coder:3b
exit /b 0
