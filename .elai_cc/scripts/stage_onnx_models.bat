@echo off
REM stage_onnx_models.bat — Windows wrapper for stage_onnx_models.py
REM Usage: stage_onnx_models.bat [--force] [--only graphcodebert,granite,...] [args]

SET "SCRIPT_DIR=%~dp0"
SET "PROJECT_ROOT=%SCRIPT_DIR%.."

IF NOT DEFINED ELAI_PROJECT_ROOT (
    SET "ELAI_PROJECT_ROOT=%PROJECT_ROOT%"
)

REM Prefer staging venv if it exists
IF EXIST "%PROJECT_ROOT%\.venv-staging\Scripts\python.exe" (
    "%PROJECT_ROOT%\.venv-staging\Scripts\python.exe" "%SCRIPT_DIR%stage_onnx_models.py" %*
) ELSE (
    python "%SCRIPT_DIR%stage_onnx_models.py" %*
)
EXIT /B %ERRORLEVEL%
