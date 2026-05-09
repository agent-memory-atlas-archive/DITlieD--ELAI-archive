@echo off
REM ELAI Anthropic OAuth login launcher
REM   Double-click to log in with your Anthropic account (claude.ai / platform.claude.com).
REM   Opens a browser to platform.claude.com/oauth/authorize, listens on
REM   http://localhost:4545/callback, exchanges the PKCE code, and saves the
REM   token to %USERPROFILE%\.elai\credentials.json under key "oauth".
REM
REM   After this completes, every elai launch (local / remote / dashboard) that
REM   hits api.anthropic.com will use the saved OAuth token automatically (auth
REM   resolution: ANTHROPIC_API_KEY -> ANTHROPIC_AUTH_TOKEN -> saved OAuth).
REM
REM   Usage:
REM     elai-anthropic.bat           -> log in (opens browser)
REM     elai-anthropic.bat logout    -> clear saved credentials

if not defined HOME set HOME=%USERPROFILE%

IF /I "%~1"=="logout" (
    "%~dp0target\release\elai.exe" logout 2>>"%~dp0elai-crash.log"
    GOTO :crashcheck
)

"%~dp0target\release\elai.exe" login 2>>"%~dp0elai-crash.log"

:crashcheck
if %ERRORLEVEL% neq 0 (
    echo.
    echo *** elai.exe exited with error ***
    echo *** stderr log: %~dp0elai-crash.log ***
    echo.
    if exist "%~dp0elai-crash.log" type "%~dp0elai-crash.log"
    echo.
    pause
) else (
    echo.
    echo Done. You can close this window.
    timeout /t 3 /nobreak ^>nul
)
