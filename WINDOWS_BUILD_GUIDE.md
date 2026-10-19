# QVCAD Windows Build Guide

## Quick Start

### Option 1: Using GitHub Actions (Recommended)
1. Push a tag to GitHub: `git tag v1.0.0 && git push origin v1.0.0`
2. GitHub Actions automatically builds both portable and installer
3. Download from Releases page

### Option 2: Manual Build on Windows

#### Prerequisites
- Windows 10 or later
- Rust (https://rustup.rs/)
- NSIS (for installer only) - `choco install nsis` or download from https://nsis.sourceforge.io/

#### Build Portable Executable

**Using PowerShell:**
```powershell
# Navigate to project directory
cd C:\path\to\OpenCADStudio

# Run build script
powershell -ExecutionPolicy Bypass -File build-windows-portable.ps1
```

**Using Batch:**
```batch
cd C:\path\to\OpenCADStudio
build-windows-portable.bat
```

**Result:**
- `releases\QVCAD-Windows-Portable-1.0.0-YYYYMMDD.exe` (~91 MB)
- `releases\QVCAD-Windows-Portable-1.0.0-YYYYMMDD.zip` (compressed)

#### Build Installer

```powershell
# Prerequisites
choco install nsis -y

# Run installer build script
powershell -ExecutionPolicy Bypass -File build-windows.ps1
```

**Result:**
- `releases\QVCAD-Windows-1.0.0-YYYYMMDD-installer.exe` (~50 MB)

### Option 3: Manual Cargo Build

```bash
# Build release binary
cargo build --release

# Binary location:
# target\release\OpenCADStudio.exe

# For portable use:
# - Copy .exe to USB drive or distribute directly
# - Users can run immediately without installation
```

## Output Files

### Portable Executable
- **File:** `QVCAD-Windows-Portable-1.0.0-YYYYMMDD.exe`
- **Size:** ~91 MB (uncompressed)
- **Type:** Standalone executable
- **Installation:** None required
- **Usage:** Run directly, USB portable, network share

### Installer
- **File:** `QVCAD-Windows-1.0.0-YYYYMMDD-installer.exe`
- **Size:** ~50 MB
- **Type:** NSIS installer
- **Installation:** Standard Windows installer experience
- **Usage:** Click to install, adds to Programs & Features

## Distribution

### Direct Distribution
```bash
# Share portable .exe
qvcad-portable.exe

# Users can:
1. Download and run immediately
2. Copy to USB drive
3. Put on network share
4. Email to colleagues
```

### Installer Package
```bash
# Share installer .exe
qvcad-installer.exe

# Users install to:
# C:\Program Files\QVCAD\
# C:\Program Files (x86)\QVCAD\  (for 32-bit)
```

### ZIP Archive
```bash
# Share portable ZIP
qvcad-portable.zip

# Users extract and run
# Works on any Windows 10+ system
```

## Build Configuration

### Portable Build Features
- ✅ No installation required
- ✅ USB portable ready
- ✅ ~91 MB standalone
- ✅ Self-contained (no dependencies)
- ✅ Windows 10+ compatible

### Installer Features
- ✅ Standard Windows installer
- ✅ Adds to Start Menu
- ✅ Adds to Programs & Features
- ✅ File associations (.dwg, .dxf)
- ✅ Uninstaller included

## Troubleshooting

### Build Fails
```bash
# Clean and rebuild
cargo clean
cargo build --release
```

### Windows Defender Warning
- Portable exe may trigger warning (no code signing)
- Users can safely ignore
- Add exception: Windows Defender > Virus & threat protection > Manage settings

### Large File Size
- 91 MB is typical for Rust GUI apps
- Contains everything needed to run
- Can be compressed with UPX if needed:
  ```bash
  upx --best --lzma qvcad-portable.exe
  ```

## System Requirements

| Component | Requirement |
|-----------|-------------|
| OS | Windows 10 or later |
| Architecture | x86-64 or ARM64 |
| RAM | 4 GB minimum |
| Disk Space | 500 MB for installation |
| GPU | Modern GPU (2010+) |

## Release Process

### Using GitHub Tags
```bash
# Create and push tag
git tag v1.0.0
git push origin v1.0.0

# GitHub Actions automatically:
# 1. Builds portable executable
# 2. Builds installer with NSIS
# 3. Creates release with both files
# 4. Uploads for download
```

### Manual Release
```bash
# Build both versions
powershell -File build-windows-portable.ps1
powershell -File build-windows.ps1

# Upload to GitHub Releases manually
# or distribute via your preferred method
```

## Next Steps

1. **Install Rust:** https://rustup.rs/
2. **Test Build:** `cargo build --release`
3. **Create Release:** Push a git tag or run build scripts
4. **Download:** Get from GitHub Releases
5. **Test:** Install on Windows 10+ system
6. **Distribute:** Share portable .exe or installer

## Support

- **Repository:** https://github.com/QUOCVIETHIEU/QVCAD
- **Issues:** Report on GitHub
- **Releases:** https://github.com/QUOCVIETHIEU/QVCAD/releases

---

**Last Updated:** October 19, 2026
