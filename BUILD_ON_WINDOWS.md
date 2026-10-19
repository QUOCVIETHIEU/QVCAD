# Build QVCAD trên Windows

## Chuẩn bị

### 1. Cài Rust
https://rustup.rs/
- Chọn "Install Now"
- Chờ cài xong

### 2. Clone project
```bash
git clone https://github.com/QUOCVIETHIEU/QVCAD.git
cd QVCAD
```

### 3. Cài dependencies (nếu có)
Mở PowerShell trong folder project

---

## Build Portable Exe (Cách Dễ Nhất)

### Option A - PowerShell (Khuyến khích)
```powershell
powershell -ExecutionPolicy Bypass -File build-windows-portable.ps1
```

### Option B - Batch File
```cmd
build-windows-portable.bat
```

### Option C - Manual Cargo
```cmd
cargo build --release
copy target\release\OpenCADStudio.exe releases\QVCAD-Windows-Portable.exe
```

---

## Kết quả

File sẽ có trong folder `releases`:
```
releases/
  ├── QVCAD-Windows-Portable-1.0.0-YYYYMMDD.exe  (~91 MB)
  └── QVCAD-Windows-Portable-1.0.0-YYYYMMDD.zip  (compressed)
```

## Double-click file .exe → App chạy ngay!

---

## Build Installer (tùy chọn)

### Cài NSIS
```cmd
choco install nsis -y
```
hoặc tải từ: https://nsis.sourceforge.io/

### Build installer
```powershell
powershell -File build-windows.ps1
```

Kết quả:
```
releases/QVCAD-Windows-1.0.0-YYYYMMDD-installer.exe
```

---

## Troubleshooting

### Lỗi: "command not found"
- Mở **PowerShell Admin** thay vì Cmd
- Hoặc dùng `build-windows-portable.bat`

### Lỗi: Rust không tìm thấy
- Restart PowerShell/Cmd
- Hoặc restart Windows

### Build quá lâu (30+ phút)
- Lần đầu tiên compile cần thời gian
- Lần sau sẽ nhanh hơn (cache)

### File size lớn (91 MB)
- Normal cho Rust GUI app
- Có thể compress với UPX nếu muốn

---

## Files cần biết

| File | Dùng |
|------|------|
| `build-windows-portable.ps1` | Build portable .exe |
| `build-windows.ps1` | Build installer |
| `build-windows-portable.bat` | Build (batch version) |
| `Cargo.toml` | Config dependencies |
| `OpenCADStudio.nsi` | NSIS installer config |

---

## Quick Build Script

Copy-paste vào PowerShell:

```powershell
# 1. Build
cargo build --release

# 2. Copy to releases folder
if (!(Test-Path "releases")) { New-Item -ItemType Directory -Path "releases" }
Copy-Item "target\release\OpenCADStudio.exe" "releases\QVCAD-Windows-Portable.exe"

# 3. Done
Write-Host "✅ Done! File: releases/QVCAD-Windows-Portable.exe"
```

---

## Done!

Bây giờ bạn có file .exe chạy được trên Windows!

**Distribute:**
- Share file .exe
- Copy vào USB
- Copy vào network share

Users chỉ cần double-click → App chạy ngay!

