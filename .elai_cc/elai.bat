@echo off
REM ELAI Local Mode — runs the full CLI against local Gemma4 E2B with DoLa + TurboQuant.
REM Routes through ConversationRuntime with FSM tool filtering + ContextBudget.

if not defined HOME set HOME=%USERPROFILE%

"%~dp0target\release\elai.exe" --local %* 2> "%~dp0elai-crash.log"
if %ERRORLEVEL% neq 0 (
    echo.
    echo === CRASH LOG ===
    type "%~dp0elai-crash.log"
    echo.
    echo Exit code: %ERRORLEVEL%
    pause
)
