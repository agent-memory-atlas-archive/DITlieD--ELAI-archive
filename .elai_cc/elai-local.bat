@echo off
REM ELAI Local Sidecar launcher
REM   Double-click to launch the ELAI dashboard via the local Python sidecar.
REM   The sidecar must be running separately (or use elai.bat which starts it).
REM   Run install-shortcuts.bat once to add desktop/Start-menu shortcuts.

if not defined HOME set HOME=%USERPROFILE%

"%~dp0target\release\elai.exe" --dashboard 2>>"%~dp0elai-crash.log"

if %ERRORLEVEL% neq 0 (
    echo.
    echo === CRASH LOG ===
    type "%~dp0elai-crash.log"
    echo.
    echo Exit code: %ERRORLEVEL%
    pause
)
