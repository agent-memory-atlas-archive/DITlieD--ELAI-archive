@echo off
REM ELAI entrypoint
REM   elai.bat            -> interactive mode picker (1-4), default=1 after 5s
REM   elai.bat --remote   -> SSH-boot remote GPU sidecar + launch dashboard
REM   elai.bat repl       -> local REPL (DoLa + TurboQuant)
REM   elai.bat ui         -> Vite dev server for dashboard-v2 (http://localhost:1420/)
REM   elai.bat local      -> dashboard via local sidecar (same as picker option 1)
REM   Requires OpenSSH in PATH and ~/.ssh/elai_remote for --remote.

if not defined HOME set HOME=%USERPROFILE%

REM Avoid auto-indexing ELAI's own binary tree. If the bat is invoked while
REM cwd == the bat's own directory (typical double-click case from inside
REM .elai_cc\), reset cwd to %USERPROFILE% so the dashboard's default
REM project_root is a harmless small directory. The user picks the real
REM project via the dashboard's project picker (open_project_from_folder).
REM If the user ran `elai.bat` from inside a real project folder, %CD% will
REM differ from %~dp0 and we leave cwd alone -- project_root = their folder.
IF /I "%CD%\"=="%~dp0" cd /d "%USERPROFILE%"

REM --- Explicit argument branches (skip picker) ---

IF /I "%~1"=="--remote" (
    "%~dp0target\release\elai.exe" --remote-sidecar 2>>"%~dp0elai-crash.log"
    GOTO :crashcheck
)

IF /I "%~1"=="repl" (
    SHIFT
    "%~dp0target\release\elai.exe" --local %* 2>>"%~dp0elai-crash.log"
    GOTO :crashcheck
)

IF /I "%~1"=="ui" (
    SHIFT
    call "%~dp0crates\dashboard-v2\start-stub-ui.bat" %*
    GOTO :eof
)

IF /I "%~1"=="local" (
    "%~dp0target\release\elai.exe" --dashboard 2>>"%~dp0elai-crash.log"
    GOTO :crashcheck
)

REM --- No-arg picker (double-click experience) ---

IF NOT "%~1"=="" GOTO :default_launch

REM E-23 first-run positioning line. Honest, cited, no marketing prose.
echo.
echo  ELAI: FSM-gated, grounded, audited code work. not a chat assistant.
echo        runtime badge (local / cloud - billed) lives in the status strip.
echo        positioning + sources documented in .elai_cc\README.md.
echo.
echo  ELAI launch mode:
echo   [1] Local sidecar  (default - dashboard via local Python sidecar)
echo   [2] Remote sidecar (friend's GPU box at 192.0.2.10 + dashboard)
echo   [3] REPL           (no dashboard - terminal interactive, local sidecar)
echo   [4] UI dev         (Vite dev server only - design preview)
echo   [5] Cloud          (Anthropic Sonnet/Opus via OAuth - no sidecar)
echo.
CHOICE /C 12345 /T 5 /D 1 /N /M "Choose mode (1-5) [default 1, 5s]: "
SET _PICK=%ERRORLEVEL%

IF %_PICK%==1 GOTO :mode_local
IF %_PICK%==2 GOTO :mode_remote
IF %_PICK%==3 GOTO :mode_repl
IF %_PICK%==4 GOTO :mode_ui
IF %_PICK%==5 GOTO :mode_cloud
GOTO :mode_local

:mode_local
"%~dp0target\release\elai.exe" --dashboard 2>>"%~dp0elai-crash.log"
GOTO :crashcheck

:mode_remote
"%~dp0target\release\elai.exe" --remote-sidecar 2>>"%~dp0elai-crash.log"
GOTO :crashcheck

:mode_repl
"%~dp0target\release\elai.exe" --local 2>>"%~dp0elai-crash.log"
GOTO :crashcheck

:mode_ui
call "%~dp0crates\dashboard-v2\start-stub-ui.bat"
GOTO :eof

:mode_cloud
REM Cloud mode: no local sidecar, Anthropic API direct via saved OAuth.
REM Sign in via the dashboard ("Sign in with Claude") if ~/.elai/credentials.json
REM is missing or the saved token has expired server-side.
SET ELAI_NO_SIDECAR=1
SET ELAI_DEFAULT_EXECUTION_MODE=CLOUD
SET ANTHROPIC_BASE_URL=
IF NOT EXIST "%USERPROFILE%\.elai\credentials.json" (
    echo.
    echo  No saved OAuth credentials at %USERPROFILE%\.elai\credentials.json.
    echo  The dashboard will prompt you to "Sign in with Claude" on launch.
    echo.
)
"%~dp0target\release\elai.exe" --dashboard 2>>"%~dp0elai-crash.log"
GOTO :crashcheck

:default_launch
"%~dp0target\release\elai.exe" --dashboard %* 2>>"%~dp0elai-crash.log"

:crashcheck
if %ERRORLEVEL% neq 0 (
    echo.
    echo === CRASH LOG ===
    type "%~dp0elai-crash.log"
    echo.
    echo Exit code: %ERRORLEVEL%
    pause
)
