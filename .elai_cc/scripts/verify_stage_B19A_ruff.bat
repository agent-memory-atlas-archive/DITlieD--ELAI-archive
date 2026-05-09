@echo off
REM B-VERIFY probe B19A_ruff wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_B19A_ruff.sh" %*
exit /b %errorlevel%
