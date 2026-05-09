@echo off
REM B-VERIFY probe B8_csq_guard wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_B8_csq_guard.sh" %*
exit /b %errorlevel%
