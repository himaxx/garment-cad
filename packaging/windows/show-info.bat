@echo off
rem Drag a Richpeace .dgs file onto this file to see sizes, pieces and fabric area.
if "%~1"=="" (
  echo Drag a .dgs pattern file onto show-info.bat
  pause
  exit /b 1
)
"%~dp0gcad.exe" info "%~1"
echo.
pause
