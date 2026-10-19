# OpenCAD Studio Release Guide

## Available Releases

### macOS
- **File**: `releases/OpenCADStudio-macOS-1.0.0-20261019.dmg`
- **Size**: 41 MB
- **Type**: DMG Installer (standard macOS format)
- **Installation**: Double-click → Drag to Applications

### Windows
Choose one of these methods:

#### Option 1: Portable Executable (Easiest)
```bash
build-windows.bat
# Output: releases/OpenCADStudio-Windows-1.0.0-*.exe
```
- Single .exe file, ~91 MB
- No installation needed
- Run directly from anywhere

#### Option 2: NSIS Installer (Professional)
```bash
# Install NSIS first: https://nsis.sourceforge.io/
# Then right-click OpenCADStudio.nsi → Compile NSI Script
# Output: releases/OpenCADStudio-Windows-1.0.0-installer.exe
```
- Professional installer UI
- Adds to Start Menu
- Creates uninstaller

#### Option 3: PowerShell
```powershell
powershell -ExecutionPolicy Bypass -File build-windows.ps1
```

## Building on Windows

### Prerequisites
1. **Rust**: https://rustup.rs/
2. **Visual Studio Build Tools**: For native compilation
3. **Git** (optional): For version control

### Quick Start
1. Open Command Prompt or PowerShell
2. Navigate to project directory
3. Run: `build-windows.bat` or `build-windows.ps1`
4. Wait for build to complete (~5-10 minutes first time)
5. Find release in `releases/` folder

### Troubleshooting

**Error: cargo not found**
- Install Rust from https://rustup.rs/

**Error: C++ build tools not found**
- Install Visual Studio Build Tools: https://visualstudio.microsoft.com/

**Slow build on first run**
- Normal! Rust compiles everything from source
- Subsequent builds are faster (incremental)

## Release Contents

Both macOS and Windows include:
- ✅ OpenCAD Studio application
- ✅ Company branding (QUOC VIET)
- ✅ MCP Configuration guide
- ✅ Performance optimizations
- ✅ All latest features

## Version Info
- **Version**: 1.0.0
- **Build Date**: 2026-10-19
- **Platform Support**: macOS, Windows, Linux (source)

## Next Steps

1. **macOS Users**: Download DMG and follow installation
2. **Windows Users**: Run build script or download pre-built
3. **Developers**: See WINDOWS_BUILD.md for detailed instructions

## Support

For issues or questions:
1. Check MCP Config button in app for setup help
2. Review WINDOWS_BUILD.md for build issues
3. Contact support at https://quocviet.com.vn
