; Build through scripts/build-release.ps1, which sets VERSION, SRC and OUTPUT.
Unicode true
!include "MUI2.nsh"
!include "x64.nsh"

!define INSTALL_KEY "Software\DragonViewer"
!define UNINSTALL_KEY "Software\Microsoft\Windows\CurrentVersion\Uninstall\DragonViewer"
!define VIEWER "dragon-viewer.exe"
!define UNINSTALLER "Uninstall Dragon Viewer.exe"
!define ICON "assets\branding\dragon-viewer.ico"

Name "Dragon Viewer ${VERSION}"
OutFile "${OUTPUT}"
InstallDir "$LOCALAPPDATA\Programs\DragonViewer"
InstallDirRegKey HKCU "${INSTALL_KEY}" "InstallDir"
RequestExecutionLevel user
SetCompressor /SOLID lzma
BrandingText "Dragon Viewer"

VIProductVersion "${VERSION}.0"
VIAddVersionKey "ProductName" "Dragon Viewer"
VIAddVersionKey "CompanyName" "Dragon Viewer"
VIAddVersionKey "FileDescription" "Dragon Viewer installer"
VIAddVersionKey "FileVersion" "${VERSION}"
VIAddVersionKey "LegalCopyright" "Dragon Viewer contributors. See NOTICE.md. GPLv3 or later."

!define MUI_ICON "..\..\assets\branding\dragon-viewer.ico"
!define MUI_UNICON "..\..\assets\branding\dragon-viewer.ico"
!define MUI_ABORTWARNING
!define MUI_WELCOMEPAGE_TITLE "Install Dragon Viewer ${VERSION}"
!define MUI_WELCOMEPAGE_TEXT "Dragon Viewer is an independent viewer for Second Life.$\r$\n$\r$\nThis software is not provided or supported by Linden Lab, the makers of Second Life.$\r$\n$\r$\nClick Next to continue."
!define MUI_FINISHPAGE_RUN "$INSTDIR\${VIEWER}"
!define MUI_FINISHPAGE_RUN_TEXT "Start Dragon Viewer"
!define MUI_FINISHPAGE_RUN_NOTCHECKED

!insertmacro MUI_PAGE_WELCOME
!insertmacro MUI_PAGE_LICENSE "..\..\LICENSE"
!insertmacro MUI_PAGE_COMPONENTS
!insertmacro MUI_PAGE_DIRECTORY
!insertmacro MUI_PAGE_INSTFILES
!insertmacro MUI_PAGE_FINISH
!insertmacro MUI_UNPAGE_CONFIRM
!insertmacro MUI_UNPAGE_INSTFILES
!insertmacro MUI_LANGUAGE "English"

Function .onInit
  ${IfNot} ${RunningX64}
    MessageBox MB_OK|MB_ICONSTOP "Dragon Viewer requires 64-bit Windows." /SD IDOK
    SetErrorLevel 1
    Abort
  ${EndIf}
FunctionEnd

Function .onVerifyInstDir
  Call CheckInstallDir
FunctionEnd

Function CheckInstallDir
  ; An existing install can be upgraded. A new install needs an empty folder.
  ReadRegStr $0 HKCU "${INSTALL_KEY}" "InstallDir"
  StrCmp $INSTDIR $0 valid
  ClearErrors
  FindFirst $0 $1 "$INSTDIR\*"
  IfErrors valid
  next:
    StrCmp $1 "" empty
    StrCmp $1 "." skip
    StrCmp $1 ".." skip
    FindClose $0
    Abort
  skip:
    FindNext $0 $1
    IfErrors empty next
  empty:
    FindClose $0
  valid:
FunctionEnd

Section "Dragon Viewer" SecMain
  SectionIn RO
  ; Also enforce the folder check for silent installs that skip the page.
  Call CheckInstallDir
  SetOutPath "$INSTDIR"
  File /r "${SRC}\*.*"
  WriteUninstaller "$INSTDIR\${UNINSTALLER}"
  CreateDirectory "$SMPROGRAMS\Dragon Viewer"
  CreateShortcut "$SMPROGRAMS\Dragon Viewer\Dragon Viewer.lnk" "$INSTDIR\${VIEWER}" "" "$INSTDIR\${ICON}"
  CreateShortcut "$SMPROGRAMS\Dragon Viewer\Uninstall Dragon Viewer.lnk" "$INSTDIR\${UNINSTALLER}" "" "$INSTDIR\${ICON}"

  WriteRegStr HKCU "${INSTALL_KEY}" "InstallDir" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayName" "Dragon Viewer"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayVersion" "${VERSION}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "Publisher" "Dragon Viewer"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "DisplayIcon" "$INSTDIR\${ICON}"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "InstallLocation" "$INSTDIR"
  WriteRegStr HKCU "${UNINSTALL_KEY}" "UninstallString" "$\"$INSTDIR\${UNINSTALLER}$\""
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoModify" 1
  WriteRegDWORD HKCU "${UNINSTALL_KEY}" "NoRepair" 1
SectionEnd

Section "Desktop shortcut" SecDesktop
  CreateShortcut "$DESKTOP\Dragon Viewer.lnk" "$INSTDIR\${VIEWER}" "" "$INSTDIR\${ICON}"
SectionEnd

!insertmacro MUI_FUNCTION_DESCRIPTION_BEGIN
  !insertmacro MUI_DESCRIPTION_TEXT ${SecMain} "Install the viewer and create Start menu shortcuts."
  !insertmacro MUI_DESCRIPTION_TEXT ${SecDesktop} "Create a Dragon Viewer shortcut on the desktop."
!insertmacro MUI_FUNCTION_DESCRIPTION_END

Section "Uninstall"
  SetOutPath "$TEMP"
  ClearErrors
  Delete "$INSTDIR\${VIEWER}"
  IfErrors 0 removeFiles
    MessageBox MB_OK|MB_ICONSTOP "Close Dragon Viewer and run the uninstaller again." /SD IDOK
    SetErrorLevel 1
    Abort
  removeFiles:
  Delete "$INSTDIR\${UNINSTALLER}"
  Delete "$INSTDIR\LICENSE"
  Delete "$INSTDIR\NOTICE.md"
  Delete "$INSTDIR\README.md"
  Delete "$INSTDIR\PRIVACY.md"
  Delete "$INSTDIR\dragon-viewer-install.json"
  Delete "$INSTDIR\${ICON}"
  Delete "$INSTDIR\assets\dictionaries\README.md"
  Delete "$INSTDIR\assets\dictionaries\*-LICENSE.txt"
  Delete "$INSTDIR\assets\emoji\LICENSE-OFL.txt"
  Delete "$INSTDIR\assets\fonts\LICENSE-*.txt"
  Delete "$INSTDIR\assets\icons\LICENSE-tabler.txt"
  Delete "$INSTDIR\assets\lsl\PREPROCESSOR-LICENSE.txt"
  Delete "$INSTDIR\assets\stars\LICENSE-HYG.md"
  Delete "$INSTDIR\assets\stars\LICENSE-TYCHO2.md"
  RMDir "$INSTDIR\assets\branding"
  RMDir "$INSTDIR\assets\dictionaries"
  RMDir "$INSTDIR\assets\emoji"
  RMDir "$INSTDIR\assets\fonts"
  RMDir "$INSTDIR\assets\icons"
  RMDir "$INSTDIR\assets\lsl"
  RMDir "$INSTDIR\assets\stars"
  RMDir "$INSTDIR\assets"
  RMDir "$INSTDIR"

  Delete "$DESKTOP\Dragon Viewer.lnk"
  Delete "$SMPROGRAMS\Dragon Viewer\Dragon Viewer.lnk"
  Delete "$SMPROGRAMS\Dragon Viewer\Uninstall Dragon Viewer.lnk"
  RMDir "$SMPROGRAMS\Dragon Viewer"
  DeleteRegKey HKCU "${UNINSTALL_KEY}"
  DeleteRegKey HKCU "${INSTALL_KEY}"
SectionEnd
