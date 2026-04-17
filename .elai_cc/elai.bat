@echo off
REM ELAI entrypoint — default launches Tauri dashboard (MVP golden path).
REM Use "elai.bat repl" to enter the local REPL (DoLa + TurboQuant on Gemma4 E2B).

if not defined HOME set HOME=%USERPROFILE%

IF /I "%~1"=="repl" (
    SHIFT
    "%~dp0target\release\elai.exe" --local %* 2> "%~dp0elai-crash.log"
) ELSE (
    "%~dp0target\release\elai.exe" --dashboard %* 2> "%~dp0elai-crash.log"
)
if %ERRORLEVEL% neq 0 (
    echo.
    echo === CRASH LOG ===
    type "%~dp0elai-crash.log"
    echo.
    echo Exit code: %ERRORLEVEL%
    pause
)
