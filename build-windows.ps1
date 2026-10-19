# OpenCADStudio Windows Build Script (PowerShell)
# Run with: powershell -ExecutionPolicy Bypass -File build-windows.ps1

Write-Host "===============================================" -ForegroundColor Cyan
Write-Host " OpenCADStudio Windows Release Builder" -ForegroundColor Cyan
Write-Host "===============================================" -ForegroundColor Cyan
Write-Host ""

$VERSION = "1.0.0"
$BUILD_DATE = Get-Date -Format "yyyyMMdd"
$RELEASE_FILE = "releases\OpenCADStudio-Windows-$VERSION-$BUILD_DATE.exe"

Write-Host "[1/4] Building release binary..." -ForegroundColor Yellow
cargo build --release

if (-not (Test-Path "target\release\OpenCADStudio.exe")) {
    Write-Host "ERROR: Build failed!" -ForegroundColor Red
    exit 1
}

Write-Host "[2/4] Creating release directory..." -ForegroundColor Yellow
if (-not (Test-Path "releases")) {
    New-Item -ItemType Directory -Path "releases" | Out-Null
}

Write-Host "[3/4] Creating portable executable..." -ForegroundColor Yellow
Copy-Item "target\release\OpenCADStudio.exe" "$RELEASE_FILE"

Write-Host "[4/4] Build complete!" -ForegroundColor Green
Write-Host ""
Write-Host "===============================================" -ForegroundColor Green
Write-Host "Release created: $RELEASE_FILE" -ForegroundColor Green
Write-Host "===============================================" -ForegroundColor Green
Write-Host ""
Write-Host "Installation options:" -ForegroundColor Cyan
Write-Host "1. Portable: Run .exe directly from anywhere"
Write-Host "2. Installed: Move to Program Files and create shortcuts"
Write-Host ""
