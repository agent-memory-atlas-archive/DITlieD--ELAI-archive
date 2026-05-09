@echo off
REM B-VERIFY probe 1G wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_1G.sh" %*
exit /b %errorlevel%
