@echo off
REM B-VERIFY probe 1I wrapper
REM graphcodebert.onnx is NOT renamed in this probe (unit test strategy).
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_1I.sh" %*
exit /b %errorlevel%
