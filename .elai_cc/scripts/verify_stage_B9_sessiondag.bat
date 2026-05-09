@echo off
REM B-VERIFY probe B9_sessiondag wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_B9_sessiondag.sh" %*
exit /b %errorlevel%
