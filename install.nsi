!include LogicLib.nsh

; Set the name and output file of the installer
Outfile "windows_x64_installer.exe"

; Set the name and version of the application
Name "Fastn"

; Version "1.0"
VIProductVersion "${VERSION}"

; Default installation directory
InstallDir $PROGRAMFILES\fastn

; Request application privileges for installation
RequestExecutionLevel admin

; Pages

Page Directory
Page InstFiles

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