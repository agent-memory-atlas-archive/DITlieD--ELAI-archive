@echo off
REM B-VERIFY probe B8_evidence_gates wrapper
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_B8_evidence_gates.sh" %*
exit /b %errorlevel%
