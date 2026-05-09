@echo off
REM ELAI Remote Sidecar launcher
REM   Double-click to boot the remote GPU sidecar (192.0.2.10) and open
REM   the dashboard with ANTHROPIC_BASE_URL=http://localhost:8100.
REM   Requires OpenSSH in PATH and ~/.ssh/elai_remote (ed25519 key).
REM   Run install-shortcuts.bat once to add desktop/Start-menu shortcuts.

if not defined HOME set HOME=%USERPROFILE%

"%~dp0target\release\elai.exe" --remote-sidecar 2>>"%~dp0elai-crash.log"
if errorlevel 1 (
    echo.
    echo *** elai.exe exited with error ***
    echo *** stderr log: %~dp0elai-crash.log ***
    echo.
    if exist "%~dp0elai-crash.log" type "%~dp0elai-crash.log"
    echo.
    pause
)
