@echo off
REM ============================================================
REM  ELAI dashboard — FULL desktop app launcher
REM
REM  Launches the real Tauri 2 + Svelte 5 desktop window from THIS
REM  directory (the live dashboard, not the web stub). Compiles the
REM  Rust Tauri shell, so first run is slow and needs the cargo
REM  toolchain on PATH.
REM
REM  For the web-only frontend (browser on :1420, no Rust backend)
REM  use `npm run dev` instead.
REM
REM  Exit: Ctrl+C, then close this window.
REM ============================================================

cd /d "%~dp0"

if not exist "node_modules\" (
  echo [start-dashboard] node_modules missing — running npm install ...
  call npm install
  if errorlevel 1 (
    echo [start-dashboard] npm install failed. aborting.
    pause
    exit /b 1
  )
)

echo [start-dashboard] launching Tauri desktop app (cwd: %~dp0) ...
call npm run tauri:dev
