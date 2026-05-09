@echo off
REM B-VERIFY probe 1D wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_1D.sh" %*
exit /b %errorlevel%
