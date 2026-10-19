# Building OpenCAD Studio for Windows

## Requirements
- Windows 10 or later
- Rust (install from https://rustup.rs/)
- Git

## Quick Build (Batch Script)

```bash
build-windows.bat
```

## Or PowerShell

```powershell
powershell -ExecutionPolicy Bypass -File build-windows.ps1
```

## Or Manual Build

```bash
cargo build --release
```

Output: `target\release\OpenCADStudio.exe`

## Creating NSIS Installer

1. Install NSIS from: https://nsis.sourceforge.io/
2. Right-click `OpenCADStudio.nsi` → Compile NSI Script
3. Output: `releases\OpenCADStudio-Windows-1.0.0-installer.exe`

## Distribution Options

### Portable (Recommended)
- Single .exe file
- No installation required
- Users can run from USB or directly

### NSIS Installer
- Professional installer
- Adds to Start Menu
- Easier uninstall
- Requires NSIS to build

## File Sizes
- Portable: ~91 MB (uncompressed)
- With UPX compression: ~30-40 MB
