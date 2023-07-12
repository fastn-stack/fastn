; Set the name and output file of the installer
Outfile "windows_x64_installer.exe"

; Set the name and version of the application
Name "Fastn"

; Set Version of installer
VIProductVersion "${VERSION}"

; Default installation directory
InstallDir $PROGRAMFILES\fastn

!define PRODUCT_NAME "fastn"
!define MUI_BRANDINGTEXT "fastn ${VERSION}"
!define MUI_ICON "fastn.ico"
!define MUI_INSTFILESPAGE_COLORS "FFFFFF 000000"
!define MUI_BGCOLOR 000000
!define MUI_TEXTCOLOR ffffff
!define MUI_FINISHPAGE_NOAUTOCLOSE
!define MUI_FINISHPAGE_SHOWREADME "https://fastn.com"
CRCCheck On

!include LogicLib.nsh

!include "MUI.nsh"

; Request application privileges for installation
RequestExecutionLevel admin

; Pages

!define MUI_WELCOMEPAGE  
;!define MUI_LICENSEPAGE
!define MUI_DIRECTORYPAGE
!define MUI_ABORTWARNING
; !define MUI_UNINSTALLER
; !define MUI_UNCONFIRMPAGE
!define MUI_FINISHPAGE 

; Sections

Section "Fastn Installer" SectionOne

    ; check for write permissions in path
    EnVar::Check "NULL" "NULL"
    Pop $0
    DetailPrint "EnVar::Check write access HKCU returned=|$0|"

    ; Set the output path for installation
    SetOutPath $INSTDIR
    
    ; Copy application files
    File "${BINARY_PATH}"

    ; Set the Path variables
    EnVar::SetHKCU
    EnVar::Check "Path" "$InstDir"
    Pop $0
    ${If} $0 = 0
    DetailPrint "Already there"
    ${Else}
    EnVar::AddValue "Path" "$InstDir"
    Pop $0 ; 0 on success
    ${EndIf}
    
SectionEnd