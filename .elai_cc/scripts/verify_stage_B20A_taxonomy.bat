@echo off
REM B-VERIFY probe B20A_taxonomy wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_B20A_taxonomy.sh" %*
exit /b %errorlevel%
