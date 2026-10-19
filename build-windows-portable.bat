@echo off
REM QVCAD Windows Portable Build Script
REM This script builds a standalone executable that runs without installation

echo.
echo ===============================================
echo  QVCAD - Windows Portable Build
echo ===============================================
echo.

setlocal enabledelayedexpansion

REM Set version
set VERSION=1.0.0
for /f "tokens=1-4 delims=/ " %%a in ('date /t') do (set BUILD_DATE=%%c%%a%%b)
set RELEASE_FILE=releases\QVCAD-Windows-Portable-%VERSION%-%BUILD_DATE%.exe
set RELEASE_ZIP=releases\QVCAD-Windows-Portable-%VERSION%-%BUILD_DATE%.zip

echo [1/5] Building release binary...
cargo build --release

if not exist "target\release\OpenCADStudio.exe" (
    echo ERROR: Build failed!
    exit /b 1
)

echo [2/5] Creating releases directory...
if not exist "releases" mkdir releases

echo [3/5] Copying portable executable...
copy target\release\OpenCADStudio.exe "%RELEASE_FILE%"

echo [4/5] Creating portable ZIP archive...
REM Create a simple ZIP with just the executable
cd releases
for /f %%A in ('dir /b "%~nx1"') do (
    REM Just the exe file for minimal size
    tar.exe -czf "%RELEASE_ZIP%" QVCAD-Windows-Portable-%VERSION%-%BUILD_DATE%.exe
)
cd ..

echo [5/5] Build complete!
echo.
echo ===============================================
echo Portable Executable Ready!
echo ===============================================
echo.
echo Files created:
echo   - %RELEASE_FILE%
echo   - %RELEASE_ZIP%
echo.
echo Usage:
echo 1. Single File Mode:
echo    Just copy the .exe and run it
echo    No installation required
echo.
echo 2. Portable USB/Drive:
echo    Extract ZIP file to USB drive
echo    Run QVCAD.exe from anywhere
echo.
echo 3. Batch Deployment:
echo    Copy .exe to user computers
echo    Users can run immediately
echo.
echo Size: ~91 MB standalone executable
echo.
pause
