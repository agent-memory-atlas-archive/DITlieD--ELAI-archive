@echo off
REM B-VERIFY probe ORCH_2A-2D wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_ORCH_2A_2D.sh" %*
exit /b %errorlevel%
