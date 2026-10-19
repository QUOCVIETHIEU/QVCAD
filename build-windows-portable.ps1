# QVCAD Windows Portable Build Script (PowerShell)
# Usage: powershell -ExecutionPolicy Bypass -File build-windows-portable.ps1

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host " QVCAD - Windows Portable Build" -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""

$VERSION = "1.0.0"
$BUILD_DATE = Get-Date -Format "yyyyMMdd"
$RELEASE_FILE = "releases\QVCAD-Windows-Portable-$VERSION-$BUILD_DATE.exe"

Write-Host "[1/5] Building release binary..." -ForegroundColor Yellow
cargo build --release

if (-not (Test-Path "target\release\OpenCADStudio.exe")) {
    Write-Host "ERROR: Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "[2/5] Creating releases directory..." -ForegroundColor Yellow
if (-not (Test-Path "releases")) {
    New-Item -ItemType Directory -Path "releases" | Out-Null
}

Write-Host "[3/5] Copying portable executable..." -ForegroundColor Yellow
Copy-Item "target\release\OpenCADStudio.exe" "$RELEASE_FILE"

Write-Host "[4/5] Creating ZIP archive..." -ForegroundColor Yellow
$ZIP_FILE = "releases\QVCAD-Windows-Portable-$VERSION-$BUILD_DATE.zip"
Compress-Archive -Path "$RELEASE_FILE" -DestinationPath "$ZIP_FILE" -Force

Write-Host "[5/5] Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "===============================================" -ForegroundColor Green
Write-Host "Portable Executable Ready!" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Files created:"
Write-Host "  - $(Split-Path $RELEASE_FILE -Leaf)"
Write-Host "  - $(Split-Path $ZIP_FILE -Leaf)"
Write-Host ""
Write-Host "Usage:" -ForegroundColor Cyan
Write-Host "1. Single File: Just run the .exe"
Write-Host "2. USB Drive: Extract ZIP and run from anywhere"
Write-Host "3. Deployment: Copy .exe to computers"
Write-Host ""
Write-Host "Size: ~91 MB standalone executable"
Write-Host ""
