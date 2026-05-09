@echo off
:: B-15A Task 9 — Entropy quarantine leak detector (Windows sibling).
::
:: EXPECTATIONS:
::   - exit 0 when no SystemTime::now / Instant::now calls are found under
::     crates\ except clock.rs and tests\.
::   - exit 1 when leaks are found; prints up to 10 leak lines for triage.
::   - --help / /? prints usage and exits 0.
::   - prefers ripgrep (rg.exe) when available, otherwise falls back to
::     PowerShell Select-String.
::
:: Status of this baseline (2026-04-26): expected to REPORT LEAKS — entropy
:: quarantine is deferred to B-15C. Non-zero exit is INFORMATIONAL, not a
:: build break.

setlocal EnableExtensions EnableDelayedExpansion

if "%~1"=="--help" goto :usage
if "%~1"=="-h"     goto :usage
if "%~1"=="/?"     goto :usage

set "SCRIPT_DIR=%~dp0"
set "CRATES_DIR=%SCRIPT_DIR%..\crates"

if not exist "%CRATES_DIR%" (
    echo check_entropy_quarantine: crates dir not found at %CRATES_DIR% 1>&2
    exit /b 2
)

set "HITS_FILE=%TEMP%\entropy_quarantine_%RANDOM%_%RANDOM%.txt"
break > "%HITS_FILE%"

where rg >nul 2>&1
if %ERRORLEVEL%==0 (
    rg --no-heading --line-number --type rust ^
        --glob "!**/clock.rs" ^
        --glob "!**/tests/**" ^
        --glob "!**/test/**" ^
        "SystemTime::now|Instant::now" "%CRATES_DIR%" > "%HITS_FILE%" 2>nul
) else (
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Get-ChildItem -Path '%CRATES_DIR%' -Recurse -Filter *.rs ^| Where-Object { $_.FullName -notmatch '\\clock\.rs$' -and $_.FullName -notmatch '\\tests?\\' } ^| Select-String -Pattern 'SystemTime::now|Instant::now' ^| ForEach-Object { '{0}:{1}:{2}' -f $_.Path, $_.LineNumber, $_.Line.TrimStart() }" > "%HITS_FILE%" 2>nul
)

set /a LEAK_COUNT=0
for /f %%A in ('type "%HITS_FILE%" ^| find /c /v ""') do set "LEAK_COUNT=%%A"

if "%LEAK_COUNT%"=="0" (
    echo check_entropy_quarantine: clean — 0 leaks.
    del /q "%HITS_FILE%" 2>nul
    exit /b 0
)

echo check_entropy_quarantine: FOUND %LEAK_COUNT% leak site(s). First 10:
powershell -NoProfile -Command "Get-Content '%HITS_FILE%' -TotalCount 10"
del /q "%HITS_FILE%" 2>nul
exit /b 1

:usage
echo Usage: check_entropy_quarantine.bat [--help^|-h^|/?]
echo.
echo Greps for std::time::{SystemTime,Instant}::now() calls in
echo .elai_cc\crates\, excluding clock.rs and tests\ subdirectories.
echo.
echo Exits 0 when clean, 1 when any leak site is found.
exit /b 0
