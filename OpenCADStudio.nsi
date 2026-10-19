; OpenCADStudio Windows Installer
; Build with: makensis OpenCADStudio.nsi

!include "MUI2.nsh"

; Basic settings
Name "OpenCAD Studio v1.0.0"
OutFile "releases\OpenCADStudio-Windows-1.0.0-installer.exe"
InstallDir "$PROGRAMFILES\OpenCADStudio"
RequestExecutionLevel admin

; Pages
!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH

!insertmacro MUI_LANGUAGE "English"

; Installation
Section "Install"
  SetOutPath "$INSTDIR"
  File "target\release\OpenCADStudio.exe"
  
  ; Create shortcuts
  CreateDirectory "$SMPROGRAMS\OpenCAD Studio"
  CreateShortcut "$SMPROGRAMS\OpenCAD Studio\OpenCAD Studio.lnk" "$INSTDIR\OpenCADStudio.exe"
  CreateShortcut "$SMPROGRAMS\OpenCAD Studio\Uninstall.lnk" "$INSTDIR\uninstall.exe"
  CreateShortcut "$DESKTOP\OpenCAD Studio.lnk" "$INSTDIR\OpenCADStudio.exe"
  
  ; Create uninstaller
  WriteUninstaller "$INSTDIR\uninstall.exe"
SectionEnd

; Uninstallation
Section "Uninstall"
  Delete "$INSTDIR\OpenCADStudio.exe"
  Delete "$INSTDIR\uninstall.exe"
  RMDir "$INSTDIR"
  
  Delete "$SMPROGRAMS\OpenCAD Studio\*.*"
  RMDir "$SMPROGRAMS\OpenCAD Studio"
  Delete "$DESKTOP\OpenCAD Studio.lnk"
SectionEnd
