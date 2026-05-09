@echo off
REM B-VERIFY probe B10_context_budget wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_B10_context_budget.sh" %*
exit /b %errorlevel%
