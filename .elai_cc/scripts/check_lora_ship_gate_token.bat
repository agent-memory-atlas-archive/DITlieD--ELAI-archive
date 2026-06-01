@echo off
:: E-47 -- LoRA ship-gate CI lint (Windows sibling).
::
:: Greps .elai_cc\elai_cc.toml for the deferral token literal
:: `DEFERRED_PER_E47_REPORT` AND `confirm_user_understanding = true`.
:: Exit 0 when BOTH are present; exit 1 otherwise.
::
:: Mirrors .elai_cc\scripts\check_anti_stale.bat structural conventions:
:: SCRIPT_DIR resolution, --help, PowerShell Select-String fallback when
:: ripgrep is not on PATH.

setlocal EnableExtensions EnableDelayedExpansion

if "%~1"=="--help" goto :usage
if "%~1"=="-h"     goto :usage
if "%~1"=="/?"     goto :usage

set "SCRIPT_DIR=%~dp0"
set "TOML_PATH=%SCRIPT_DIR%..\elai_cc.toml"

if not exist "%TOML_PATH%" (
    echo check_lora_ship_gate_token: elai_cc.toml not found at %TOML_PATH% 1>&2
    exit /b 1
)

where rg >nul 2>&1
set "HAVE_RG=%ERRORLEVEL%"

set "TMP_TOKEN=%TEMP%\lora_ship_gate_token_%RANDOM%_%RANDOM%.txt"
set "TMP_CONFIRM=%TEMP%\lora_ship_gate_confirm_%RANDOM%_%RANDOM%.txt"
break > "%TMP_TOKEN%"
break > "%TMP_CONFIRM%"

if "%HAVE_RG%"=="0" (
    rg --fixed-strings "ship_gate = \"DEFERRED_PER_E47_REPORT\"" "%TOML_PATH%" > "%TMP_TOKEN%" 2>nul
    rg --fixed-strings "confirm_user_understanding = true" "%TOML_PATH%" > "%TMP_CONFIRM%" 2>nul
) else (
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Select-String -Path '%TOML_PATH%' -SimpleMatch -Pattern 'ship_gate = \"DEFERRED_PER_E47_REPORT\"' ^| ForEach-Object { $_.Line }" > "%TMP_TOKEN%" 2>nul
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Select-String -Path '%TOML_PATH%' -SimpleMatch -Pattern 'confirm_user_understanding = true' ^| ForEach-Object { $_.Line }" > "%TMP_CONFIRM%" 2>nul
)

set /a TOKEN_COUNT=0
for /f %%A in ('type "%TMP_TOKEN%" ^| find /c /v ""') do set "TOKEN_COUNT=%%A"
set /a CONFIRM_COUNT=0
for /f %%A in ('type "%TMP_CONFIRM%" ^| find /c /v ""') do set "CONFIRM_COUNT=%%A"

echo check_lora_ship_gate_token: E-47 LoRA ship-gate token audit
echo   ship_gate token  : %TOKEN_COUNT% match(es) -- expected ^>=1
echo   user_confirm flag: %CONFIRM_COUNT% match(es) -- expected ^>=1

set "FAIL=0"
if "%TOKEN_COUNT%"=="0" (
    echo FAIL: 'ship_gate = "DEFERRED_PER_E47_REPORT"' missing from %TOML_PATH% 1>&2
    set "FAIL=1"
)
if "%CONFIRM_COUNT%"=="0" (
    echo FAIL: 'confirm_user_understanding = true' missing from %TOML_PATH% 1>&2
    set "FAIL=1"
)

del /q "%TMP_TOKEN%" "%TMP_CONFIRM%" 2>nul

if "%FAIL%"=="0" (
    echo check_lora_ship_gate_token: PASS -- default-deferred posture intact.
    exit /b 0
)

echo.
echo check_lora_ship_gate_token: FAIL -- gate semantic drift detected.
echo Flip per workflow\decisions\E-47-flip-runbook.md or restore the deferral token.
exit /b 1

:usage
echo Usage: check_lora_ship_gate_token.bat [--help^|-h^|/?]
echo.
echo Greps .elai_cc\elai_cc.toml for the E-47 LoRA ship-gate deferral
echo token and user-understanding confirmation.
echo.
echo Exit 0: BOTH ship_gate = "DEFERRED_PER_E47_REPORT" AND
echo         confirm_user_understanding = true present in elai_cc.toml.
echo Exit 1: either marker absent or renamed.
exit /b 0
