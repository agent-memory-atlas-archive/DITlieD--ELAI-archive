@echo off
REM B-VERIFY probe 1A wrapper — calls bash variant via Git Bash
REM Usage: verify_stage_1A.bat [corpus_root]
set SCRIPT_DIR=%~dp0
bash "%SCRIPT_DIR%verify_stage_1A.sh" %*
exit /b %errorlevel%
