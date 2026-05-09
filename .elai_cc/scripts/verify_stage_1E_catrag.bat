@echo off
REM B-VERIFY probe 1E_catrag wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_1E_catrag.sh" %*
exit /b %errorlevel%
