@echo off
REM B-VERIFY aggregator wrapper for Windows cmd.exe
REM Calls the bash aggregator via Git Bash.
REM Usage: run_b_verify.bat [--skip-sidecar] [--skip-elai-run] [--skip-build]
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%run_b_verify.sh" %*
exit /b %errorlevel%
