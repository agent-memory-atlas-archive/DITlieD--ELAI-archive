@echo off
REM B-VERIFY probe B19A_stop_hook wrapper
REM NOTE: OS-signal simulation not used; relies on unit tests only.
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_B19A_stop_hook.sh" %*
exit /b %errorlevel%
