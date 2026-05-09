@echo off
REM B-VERIFY probe 1E_arag wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_1E_arag.sh" %*
exit /b %errorlevel%
