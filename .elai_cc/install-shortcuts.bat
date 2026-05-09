@echo off
REM ELAI Shortcut Installer - run this once after install to add
REM desktop and Start-menu shortcuts for ELAI Local and ELAI Remote.

setlocal EnableExtensions

SET _DIR=%~dp0
REM Strip trailing backslash for use in PowerShell strings
SET _DIR=%_DIR:~0,-1%

SET _EXE=%_DIR%\target\release\elai.exe
SET _REMOTE_BAT=%_DIR%\elai-remote.bat
SET _LOCAL_BAT=%_DIR%\elai-local.bat
SET _ICON=%_EXE%

echo Creating ELAI desktop and Start-menu shortcuts...

powershell.exe -NoProfile -ExecutionPolicy Bypass -Command ^
  "$shell = New-Object -ComObject WScript.Shell; " ^
  "$desktop = [Environment]::GetFolderPath('Desktop'); " ^
  "$startMenu = [Environment]::GetFolderPath('StartMenu') + '\Programs'; " ^
  "$lnkPaths = @($desktop, $startMenu); " ^
  "foreach ($dir in $lnkPaths) { " ^
    "$lnk = $shell.CreateShortcut($dir + '\ELAI (Local).lnk'); " ^
    "$lnk.TargetPath  = '%_LOCAL_BAT%'; " ^
    "$lnk.WorkingDirectory = '%_DIR%'; " ^
    "$lnk.IconLocation = '%_ICON%,0'; " ^
    "$lnk.Description = 'ELAI - local sidecar dashboard'; " ^
    "$lnk.Save(); " ^
    "$lnk2 = $shell.CreateShortcut($dir + '\ELAI (Remote).lnk'); " ^
    "$lnk2.TargetPath  = '%_REMOTE_BAT%'; " ^
    "$lnk2.WorkingDirectory = '%_DIR%'; " ^
    "$lnk2.IconLocation = '%_ICON%,0'; " ^
    "$lnk2.Description = 'ELAI - remote GPU sidecar dashboard'; " ^
    "$lnk2.Save(); " ^
  "}"

if %ERRORLEVEL%==0 (
    echo Done. Shortcuts created on Desktop and in Start Menu ^> Programs.
) else (
    echo ERROR: shortcut creation failed ^(exit %ERRORLEVEL%^).
    echo Try running this script as Administrator if shortcuts are in Start Menu.
    pause
)

endlocal
