@echo off
:: B-15A Task 10 — Determinism proof harness (Windows sibling).
::
:: EXPECTATIONS:
::   - runs `cargo test -p elai-orchestrator --test bsp_determinism` twice
::     with a fixed seed, captures stdout, and asserts SHA256-identical
::     filtered output.
::   - exit 0 on match, 1 on divergence or test failure.
::   - --help / /? prints usage and exits 0.
::   - ELAI_DETERMINISM_SEED env var (default 0xB15A) selects the seed.

setlocal EnableExtensions EnableDelayedExpansion

if "%~1"=="--help" goto :usage
if "%~1"=="-h"     goto :usage
if "%~1"=="/?"     goto :usage

set "SCRIPT_DIR=%~dp0"
set "CRATE_ROOT=%SCRIPT_DIR%.."
if "%ELAI_DETERMINISM_SEED%"=="" set "ELAI_DETERMINISM_SEED=0xB15A"

set "OUT_A=%TEMP%\determinism_a_%RANDOM%_%RANDOM%.txt"
set "OUT_B=%TEMP%\determinism_b_%RANDOM%_%RANDOM%.txt"

echo determinism_proof: run 1 with seed=%ELAI_DETERMINISM_SEED%
pushd "%CRATE_ROOT%" >nul
cargo test -p elai-orchestrator --test bsp_determinism -- --nocapture > "%OUT_A%" 2>&1
set "RC1=%ERRORLEVEL%"
popd >nul
if not "%RC1%"=="0" (
    echo determinism_proof: cargo test run 1 failed with code %RC1%; output: 1>&2
    type "%OUT_A%" 1>&2
    del /q "%OUT_A%" "%OUT_B%" 2>nul
    exit /b 1
)

echo determinism_proof: run 2 with seed=%ELAI_DETERMINISM_SEED%
pushd "%CRATE_ROOT%" >nul
cargo test -p elai-orchestrator --test bsp_determinism -- --nocapture > "%OUT_B%" 2>&1
set "RC2=%ERRORLEVEL%"
popd >nul
if not "%RC2%"=="0" (
    echo determinism_proof: cargo test run 2 failed with code %RC2%; output: 1>&2
    type "%OUT_B%" 1>&2
    del /q "%OUT_A%" "%OUT_B%" 2>nul
    exit /b 1
)

:: Filter timing / "running N tests" lines and SHA256 the rest.
powershell -NoProfile -ExecutionPolicy Bypass -Command ^
    "$pat = '(finished in|running [0-9]+ test)'; ^
     function H($p){ $lines = Get-Content -LiteralPath $p ^| Where-Object { $_ -notmatch $pat } ^| ForEach-Object { ($_ -replace '[0-9]+ms','Nms') -replace '[0-9]+\.[0-9]+s','N.Ns' } ; $bytes = [System.Text.Encoding]::UTF8.GetBytes(($lines -join \"`n\")); $sha = [System.Security.Cryptography.SHA256]::Create(); ($sha.ComputeHash($bytes) ^| ForEach-Object { $_.ToString('x2') }) -join '' } ; ^
     $a = H('%OUT_A%'); $b = H('%OUT_B%'); ^
     Write-Output ('determinism_proof: hash_a=' + $a); ^
     Write-Output ('determinism_proof: hash_b=' + $b); ^
     if ($a -eq $b) { Write-Output 'determinism_proof: PASS'; exit 0 } else { Write-Error 'determinism_proof: FAIL — runs diverged.'; exit 1 }"

set "RC=%ERRORLEVEL%"
del /q "%OUT_A%" "%OUT_B%" 2>nul
exit /b %RC%

:usage
echo Usage: determinism_proof.bat [--help^|-h^|/?]
echo.
echo Runs the orchestrator bsp_determinism test twice with a fixed seed and
echo asserts byte-identical filtered output across runs.
echo.
echo Environment:
echo   ELAI_DETERMINISM_SEED  fixed seed (default: 0xB15A)
exit /b 0
