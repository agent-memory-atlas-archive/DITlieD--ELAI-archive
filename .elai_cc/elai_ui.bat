@echo off
REM Top-level shortcut: launch the dashboard-v2 stub UI (Vite dev
REM server + browser tab on http://localhost:1420). Delegates to
REM crates\dashboard-v2\start-stub-ui.bat so the launch logic stays
REM in one place.

call "%~dp0crates\dashboard-v2\start-stub-ui.bat" %*
