@echo off
:: B-16A Task 14 — Anti-stale pattern lint (Windows sibling).
::
:: STRICT (exit 1 on hit):  Box<dyn .*Error  ^|  use log::  ^|  ort = "...-rc..."
:: REPORT-ONLY (count): .unwrap() ^| as i*/u*/f*
::
:: Excludes: *\tests\*, *\test\*, *\examples\*, target\, main.rs, build.rs.
::
:: Status (2026-04-26): expected to be STRICT-clean for elai-knowledge after
:: B-16A. ort -rc is WAIVED via workflow\reasoning\B-16A-decisions.md
:: (DECISION-A) until ort 2.0.0 stable is published on crates.io.

setlocal EnableExtensions EnableDelayedExpansion

if "%~1"=="--help" goto :usage
if "%~1"=="-h"     goto :usage
if "%~1"=="/?"     goto :usage

set "SCRIPT_DIR=%~dp0"
set "CRATES_DIR=%SCRIPT_DIR%..\crates"

if not exist "%CRATES_DIR%" (
    echo check_anti_stale: crates dir not found at %CRATES_DIR% 1>&2
    exit /b 2
)

set "TMP_BOX=%TEMP%\anti_stale_box_%RANDOM%_%RANDOM%.txt"
set "TMP_LOG=%TEMP%\anti_stale_log_%RANDOM%_%RANDOM%.txt"
set "TMP_ORT=%TEMP%\anti_stale_ort_%RANDOM%_%RANDOM%.txt"
set "TMP_UNWRAP=%TEMP%\anti_stale_unwrap_%RANDOM%_%RANDOM%.txt"
set "TMP_CAST=%TEMP%\anti_stale_cast_%RANDOM%_%RANDOM%.txt"
break > "%TMP_BOX%"
break > "%TMP_LOG%"
break > "%TMP_ORT%"
break > "%TMP_UNWRAP%"
break > "%TMP_CAST%"

where rg >nul 2>&1
set "HAVE_RG=%ERRORLEVEL%"

if "%HAVE_RG%"=="0" (
    rg --no-heading --line-number --type rust ^
        --glob "!**/tests/**" --glob "!**/test/**" ^
        --glob "!**/examples/**" --glob "!**/target/**" ^
        --glob "!**/main.rs" --glob "!**/build.rs" ^
        "Box<dyn [A-Za-z_:]*Error[ +>]" "%CRATES_DIR%" > "%TMP_BOX%" 2>nul

    rg --no-heading --line-number --type rust ^
        --glob "!**/tests/**" --glob "!**/test/**" ^
        --glob "!**/examples/**" --glob "!**/target/**" ^
        --glob "!**/main.rs" --glob "!**/build.rs" ^
        "^use log::|[^a-zA-Z_]use log::" "%CRATES_DIR%" > "%TMP_LOG%" 2>nul

    rg --no-heading --line-number ^
        --glob "**/Cargo.toml" --glob "!**/target/**" ^
        "^ort\s*=\s*\"[^\"]*-rc" "%CRATES_DIR%" > "%TMP_ORT%" 2>nul

    rg --no-heading --line-number --type rust ^
        --glob "!**/tests/**" --glob "!**/test/**" ^
        --glob "!**/examples/**" --glob "!**/target/**" ^
        --glob "!**/main.rs" --glob "!**/build.rs" ^
        "\.unwrap\(\)" "%CRATES_DIR%" > "%TMP_UNWRAP%" 2>nul

    rg --no-heading --line-number --type rust ^
        --glob "!**/tests/**" --glob "!**/test/**" ^
        --glob "!**/examples/**" --glob "!**/target/**" ^
        --glob "!**/main.rs" --glob "!**/build.rs" ^
        " as (i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64)([^a-zA-Z0-9_]|$)" ^
        "%CRATES_DIR%" > "%TMP_CAST%" 2>nul
) else (
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Get-ChildItem -Path '%CRATES_DIR%' -Recurse -Filter *.rs ^| Where-Object { $_.FullName -notmatch '\\tests?\\' -and $_.FullName -notmatch '\\examples\\' -and $_.FullName -notmatch '\\target\\' -and $_.Name -ne 'main.rs' -and $_.Name -ne 'build.rs' } ^| Select-String -Pattern 'Box<dyn [A-Za-z_:]*Error[ +>]' ^| ForEach-Object { '{0}:{1}:{2}' -f $_.Path, $_.LineNumber, $_.Line.TrimStart() }" > "%TMP_BOX%" 2>nul
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Get-ChildItem -Path '%CRATES_DIR%' -Recurse -Filter *.rs ^| Where-Object { $_.FullName -notmatch '\\tests?\\' -and $_.FullName -notmatch '\\examples\\' -and $_.FullName -notmatch '\\target\\' -and $_.Name -ne 'main.rs' -and $_.Name -ne 'build.rs' } ^| Select-String -Pattern '^use log::|[^a-zA-Z_]use log::' ^| ForEach-Object { '{0}:{1}:{2}' -f $_.Path, $_.LineNumber, $_.Line.TrimStart() }" > "%TMP_LOG%" 2>nul
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Get-ChildItem -Path '%CRATES_DIR%' -Recurse -Filter Cargo.toml ^| Where-Object { $_.FullName -notmatch '\\target\\' } ^| Select-String -Pattern '^ort\s*=\s*\"[^\"]*-rc' ^| ForEach-Object { '{0}:{1}:{2}' -f $_.Path, $_.LineNumber, $_.Line.TrimStart() }" > "%TMP_ORT%" 2>nul
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Get-ChildItem -Path '%CRATES_DIR%' -Recurse -Filter *.rs ^| Where-Object { $_.FullName -notmatch '\\tests?\\' -and $_.FullName -notmatch '\\examples\\' -and $_.FullName -notmatch '\\target\\' -and $_.Name -ne 'main.rs' -and $_.Name -ne 'build.rs' } ^| Select-String -Pattern '\.unwrap\(\)' ^| ForEach-Object { '{0}:{1}:{2}' -f $_.Path, $_.LineNumber, $_.Line.TrimStart() }" > "%TMP_UNWRAP%" 2>nul
    powershell -NoProfile -ExecutionPolicy Bypass -Command ^
        "Get-ChildItem -Path '%CRATES_DIR%' -Recurse -Filter *.rs ^| Where-Object { $_.FullName -notmatch '\\tests?\\' -and $_.FullName -notmatch '\\examples\\' -and $_.FullName -notmatch '\\target\\' -and $_.Name -ne 'main.rs' -and $_.Name -ne 'build.rs' } ^| Select-String -Pattern ' as (i8|i16|i32|i64|i128|u8|u16|u32|u64|u128|f32|f64)([^a-zA-Z0-9_]|$)' ^| ForEach-Object { '{0}:{1}:{2}' -f $_.Path, $_.LineNumber, $_.Line.TrimStart() }" > "%TMP_CAST%" 2>nul
)

:: Filter out doc comments (/// or //!) and rules/src/schema.rs (deprecated_pattern fixture registry)
set "TMP_BOX_F=%TEMP%\anti_stale_box_f_%RANDOM%_%RANDOM%.txt"
break > "%TMP_BOX_F%"
powershell -NoProfile -Command "Get-Content '%TMP_BOX%' ^| Where-Object { $_ -notmatch 'rules[\\\\/]+src[\\\\/]+schema\.rs:' -and $_ -notmatch ':\s*///' -and $_ -notmatch ':\s*//!' } ^| Set-Content '%TMP_BOX_F%'"
copy /y "%TMP_BOX_F%" "%TMP_BOX%" >nul
del /q "%TMP_BOX_F%" 2>nul

set /a BOX_COUNT=0
for /f %%A in ('type "%TMP_BOX%" ^| find /c /v ""') do set "BOX_COUNT=%%A"
set /a LOG_COUNT=0
for /f %%A in ('type "%TMP_LOG%" ^| find /c /v ""') do set "LOG_COUNT=%%A"
set /a ORT_COUNT=0
for /f %%A in ('type "%TMP_ORT%" ^| find /c /v ""') do set "ORT_COUNT=%%A"
set /a UNWRAP_COUNT=0
for /f %%A in ('type "%TMP_UNWRAP%" ^| find /c /v ""') do set "UNWRAP_COUNT=%%A"
set /a CAST_COUNT=0
for /f %%A in ('type "%TMP_CAST%" ^| find /c /v ""') do set "CAST_COUNT=%%A"

echo check_anti_stale: anti-stale pattern audit (B-16A / B-16E categories)
echo   STRICT  Box^<dyn .*Error : %BOX_COUNT%
echo   STRICT  use log::       : %LOG_COUNT%
echo   STRICT  ort = "...-rc...": %ORT_COUNT%  (waiver: workflow\reasoning\B-16A-decisions.md DECISION-A; ORT-PENDING: bump to 2.0.0 when crates.io ships stable)
echo   REPORT  .unwrap()       : %UNWRAP_COUNT% (B-16B Chunks A+B+D shipped; Chunk C closed via LazyLock-static exemption per B-16B D5)
echo   REPORT  as i*/u*/f*     : %CAST_COUNT% (B-16D/E categories tracked; see check_anti_stale.sh for split)
echo   REPORT  magic-numbers   : enforced by T2 AST lint, NOT this grep script
echo     surface: crates\plugins\src\tiered_analysis.rs:1458 check_magic_numbers_rust
echo     surface: crates\plugins\src\tiered_analysis.rs:1506 check_magic_numbers_python
echo     walker : tiered_analysis.rs:1471 walk_magic_numbers_rust / :1519 walk_magic_numbers_python
echo     allow-list: 0 ^| 1 ^| 2 ^| -1 ^| 0.0 ^| 1.0 ; ctx-exempt: const/static decls + test files
echo     rationale: tree-sitter AST exempts const/static blocks deterministically; bash grep cannot

set "WAIVER=%SCRIPT_DIR%..\..\workflow\reasoning\B-16A-decisions.md"
set "ORT_FAIL=0"
if not %ORT_COUNT%==0 (
    if not exist "%WAIVER%" set "ORT_FAIL=1"
)

set "STRICT_FAIL=0"
if not %BOX_COUNT%==0 (
    set "STRICT_FAIL=1"
    echo.
    echo STRICT: Box^<dyn Error^> hits ^(first 10^):
    powershell -NoProfile -Command "Get-Content '%TMP_BOX%' -TotalCount 10"
)
if not %LOG_COUNT%==0 (
    set "STRICT_FAIL=1"
    echo.
    echo STRICT: 'use log::' hits ^(first 10^):
    powershell -NoProfile -Command "Get-Content '%TMP_LOG%' -TotalCount 10"
)
if "%ORT_FAIL%"=="1" (
    set "STRICT_FAIL=1"
    echo.
    echo STRICT: ort -rc pin without waiver doc ^(first 10^):
    powershell -NoProfile -Command "Get-Content '%TMP_ORT%' -TotalCount 10"
)

del /q "%TMP_BOX%" "%TMP_LOG%" "%TMP_ORT%" "%TMP_UNWRAP%" "%TMP_CAST%" 2>nul

if "%STRICT_FAIL%"=="0" (
    echo.
    echo check_anti_stale: STRICT clean. REPORT counts above are informational.
    exit /b 0
)

echo.
echo check_anti_stale: STRICT violations found. Fix or document waiver.
exit /b 1

:usage
echo Usage: check_anti_stale.bat [--help^|-h^|/?]
echo.
echo Greps for outdated/forbidden patterns under .elai_cc\crates\.
echo STRICT (exit 1):  Box^<dyn .*Error  ^|  use log::  ^|  ort = "...-rc..."
echo REPORT-ONLY:      .unwrap^(^), narrowing casts (B-16B/D will tighten)
echo Excludes: *\tests\*, *\test\*, *\examples\*, target\, main.rs, build.rs.
exit /b 0
