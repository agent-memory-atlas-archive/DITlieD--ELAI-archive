@echo off
REM B-VERIFY probe 1C wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_1C.sh" %*
exit /b %errorlevel%
