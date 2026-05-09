@echo off
REM B-VERIFY probe 1B wrapper — calls bash variant via Git Bash
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_1B.sh" %*
exit /b %errorlevel%
