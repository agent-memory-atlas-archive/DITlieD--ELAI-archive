@echo off
REM B-VERIFY probe B20A_similarity wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_B20A_similarity.sh" %*
exit /b %errorlevel%
