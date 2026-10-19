@echo off
REM OpenCADStudio Windows Build Script
REM Run this on Windows with Rust installed (https://rustup.rs/)

echo.
echo ===============================================
echo  OpenCADStudio Windows Release Builder
echo ===============================================
echo.

REM Set version
set VERSION=1.0.0
for /f "tokens=1-4 delims=/ " %%a in ('date /t') do (set BUILD_DATE=%%c%%a%%b)
set RELEASE_FILE=releases\OpenCADStudio-Windows-%VERSION%-%BUILD_DATE%.exe

echo [1/4] Building release binary...
cargo build --release

if not exist "target\release\OpenCADStudio.exe" (
    echo ERROR: Build failed!
    exit /b 1
)

echo [2/4] Creating release directory...
if not exist "releases" mkdir releases

echo [3/4] Creating portable installer...
REM Copy binary to releases folder
copy target\release\OpenCADStudio.exe "%RELEASE_FILE%"

echo [4/4] Build complete!
echo.
echo ===============================================
echo Release created: %RELEASE_FILE%
echo Size: [check file properties]
echo ===============================================
echo.
echo Installation:
echo 1. Users can run the .exe directly
echo 2. Or move to Program Files\OpenCADStudio
echo 3. Create shortcut to Start Menu if needed
echo.
pause
