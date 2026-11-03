@echo off
rem Drag a Richpeace .dgs file onto this file.
rem Creates, next to the pattern file:
rem   <name>_all_sizes.svg  - preview of all sizes (open in Chrome/Edge)
rem   <name>_base.plt       - plotter file of the base size (open in PlotCenter)
if "%~1"=="" (
  echo Drag a .dgs pattern file onto make-plot.bat
  pause
  exit /b 1
)
"%~dp0gcad.exe" svg "%~1" -o "%~dpn1_all_sizes.svg" --size all
"%~dp0gcad.exe" plt "%~1" -o "%~dpn1_base.plt"
echo.
echo Done. Files are in: %~dp1
pause
