# QVCAD Windows - Click & Run Portable

## Đây là cái bạn cần! 👇

### File: `QVCAD-Windows-Portable-*.exe`

**Nhấn vào là chạy ngay:**
1. Download file `.exe`
2. Double-click file
3. App chạy luôn (không cần cài đặt)

**Không cần:**
- ❌ Cài đặt
- ❌ Admin rights
- ❌ Dependencies
- ❌ Reboot

**Có thể làm:**
- ✅ Chạy từ USB drive
- ✅ Chạy từ Downloads folder
- ✅ Chạy từ Desktop
- ✅ Chạy từ Network drive

---

## Cách Build (trên Windows)

### Cách 1: PowerShell (Dễ nhất)
```powershell
powershell -ExecutionPolicy Bypass -File build-windows-portable.ps1
```

### Cách 2: Batch File
```cmd
build-windows-portable.bat
```

### Cách 3: GitHub Actions (Tự động)
```bash
git tag v1.0.0
git push origin v1.0.0
# GitHub tự build, tải về từ Releases
```

---

## File Output

**File portable:**
- Name: `QVCAD-Windows-Portable-1.0.0-YYYYMMDD.exe`
- Size: ~91 MB
- Usage: Click & Run
- Portable: USB drive friendly

**File ZIP:**
- Name: `QVCAD-Windows-Portable-1.0.0-YYYYMMDD.zip`
- Size: ~30 MB (compressed)
- Usage: Extract .exe then run

---

## Phân phối

**Cách 1: Trực tiếp**
- Share file .exe
- Users tải về
- Nhấn vào là chạy

**Cách 2: USB Drive**
- Copy .exe vào USB
- Users cắm USB
- Nhấn vào là chạy

**Cách 3: Network Share**
- Copy .exe vào network
- Users mở folder
- Nhấn vào là chạy

---

## System Requirements

- Windows 10+
- 500MB free space
- Modern GPU (for CAD)

That's it! 🚀
