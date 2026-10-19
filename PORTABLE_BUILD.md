# QVCAD Windows Portable Build Guide

## What is Portable?

A portable executable is a standalone .exe file that:
- ✅ Runs without installation
- ✅ No admin rights required
- ✅ No dependencies to install
- ✅ Can run from USB drive
- ✅ Works on any Windows 10+ machine
- ✅ No registry entries

## Building Portable Version

### Option 1: Batch Script (Easiest)
```bash
build-windows-portable.bat
```

### Option 2: PowerShell
```powershell
powershell -ExecutionPolicy Bypass -File build-windows-portable.ps1
```

### Option 3: Manual
```bash
cargo build --release
copy target\release\OpenCADStudio.exe releases\QVCAD-Windows-Portable-1.0.0-*.exe
```

## Output

Both scripts create:
- **QVCAD-Windows-Portable-1.0.0-YYYYMMDD.exe** (~91 MB)
- **QVCAD-Windows-Portable-1.0.0-YYYYMMDD.zip** (compressed)

## Deployment Options

### Option A: Direct Distribution
1. Share .exe file via email/download link
2. Users download and run
3. No installation needed

### Option B: USB Portable
1. Create folder: `D:\QVCAD\`
2. Copy QVCAD.exe to folder
3. Users plug in USB and run
4. Works on any Windows machine

### Option C: Company Deployment
1. Distribute .exe to network
2. Users run from network share
3. No installation required
4. Auto-update capable

## System Requirements

- Windows 10 or later
- 100 MB free disk space
- No additional software needed

## Build Requirements

- Rust (https://rustup.rs/)
- Windows 10+ system
- ~5-10 minutes build time

## File Size

- Uncompressed: ~91 MB
- With UPX compression: ~30-40 MB (optional)

## Compression (Optional)

To compress with UPX:
```bash
upx --best --lzma releases\QVCAD-Windows-Portable-1.0.0-*.exe
```

This reduces size to ~30-40 MB but increases startup time slightly.

## Troubleshooting

**Windows Defender Warning:**
- Code signing not included in portable version
- Users can safely ignore the warning
- Add exception in Windows Defender if needed

**Large File Size:**
- 91 MB is typical for Rust GUI apps
- Consider UPX compression for distribution
- Uncompressed startup is faster

**Dependencies Not Found:**
- Verify Visual C++ Redistributable is installed
- Or build with static linking: `RUSTFLAGS=-C target-feature=+crt-static`
