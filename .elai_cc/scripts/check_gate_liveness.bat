@echo off
:: B-17 Task 1 helper — Gate liveness regression runner (Windows sibling).
::
:: Mirrors check_gate_liveness.sh: cargo test -p elai-orchestrator --test
:: gate_liveness, parse the result line, exit 0 / 1 / 2.
::
:: `#[ignore]` stubs for B-2 / B-4 / B-6 are expected and do not fail the run.

setlocal EnableExtensions EnableDelayedExpansion

if "%~1"=="--help" goto :usage
if "%~1"=="-h"     goto :usage
if "%~1"=="/?"     goto :usage

set "SCRIPT_DIR=%~dp0"
set "CRATES_DIR=%SCRIPT_DIR%..\crates"
set "MANIFEST=%CRATES_DIR%\orchestrator\Cargo.toml"

if not exist "%MANIFEST%" (
    echo check_gate_liveness: manifest not found at %MANIFEST% 1>&2
    exit /b 2
)

set "OUT=%TEMP%\gate_liveness_out_%RANDOM%_%RANDOM%.txt"
break > "%OUT%"

cargo test --manifest-path "%MANIFEST%" --test gate_liveness -- --nocapture > "%OUT%" 2>&1
set "RC=%ERRORLEVEL%"

:: Echo the captured output verbatim for transparency.
type "%OUT%"

:: Parse "test result: ok. N passed; M failed; K ignored; ..." line.
set "SUMMARY_LINE="
for /f "usebackq delims=" %%L in (`findstr /R "^test result:" "%OUT%"`) do set "SUMMARY_LINE=%%L"

if "%SUMMARY_LINE%"=="" (
    echo.
    echo check_gate_liveness: could not find 'test result:' line in cargo output 1>&2
    del /q "%OUT%" 2>nul
    exit /b 1
)

set "PASSED=0"
set "FAILED=0"
set "IGNORED=0"

:: Primary parse: positional tokens on "test result: ok. N passed; M failed; K ignored; ..."
:: Tokens: 1=test 2=result: 3=ok.|FAILED. 4=N 5=passed; 6=M 7=failed; 8=K 9=ignored;
for /f "tokens=1-9" %%A in ("%SUMMARY_LINE%") do (
    if "%%C"=="ok." (
        set "PASSED=%%D"
        set "FAILED=%%F"
        set "IGNORED=%%H"
    ) else if "%%C"=="FAILED." (
        set "PASSED=%%D"
        set "FAILED=%%F"
        set "IGNORED=%%H"
    )
)

:: Fallback parse: if primary left PASSED=0 AND cargo exited 0, extract via findstr /R
:: digit patterns. Handles minor cargo output format changes (e.g., extra spaces, bold).
if "%PASSED%"=="0" if "%RC%"=="0" (
    for /f "tokens=1" %%N in ('echo %SUMMARY_LINE% ^| findstr /R "[0-9][0-9]* passed"') do set "PASSED=%%N"
)

echo.
echo check_gate_liveness: B-17 gate-liveness regression suite
echo   passed : %PASSED%
echo   failed : %FAILED%
echo   ignored: %IGNORED%  (wiring-blocked stubs for B-2/B-4/B-6)

del /q "%OUT%" 2>nul

if not "%RC%"=="0" (
    echo.
    echo check_gate_liveness: cargo test exited %RC% -- at least one wired gate stopped firing. 1>&2
    exit /b 1
)

if not "%FAILED%"=="0" (
    echo.
    echo check_gate_liveness: %FAILED% failed test^(s^) -- gate liveness regression. 1>&2
    exit /b 1
)

echo.
echo check_gate_liveness: clean. All wired gates fired; %IGNORED% stub^(s^) pending wiring.
exit /b 0

:usage
echo Usage: check_gate_liveness.bat [--help^|-h^|/?]
echo.
echo Runs the B-17 gate-liveness regression suite scoped to elai-orchestrator.
echo Exits 0 on clean pass, 1 on any failure, 2 on environment error.
exit /b 0
