; Set the name and output file of the installer
Outfile "fastn_installer.exe"

; Set the name and version of the application
Name "Fastn"
; Version "1.0"
VIProductVersion "1.0"

; Default installation directory
InstallDir $PROGRAMFILES\fastn

; Request application privileges for installation
RequestExecutionLevel admin

; Pages

Page Directory
Page InstFiles

; Sections

Section "Fastn Installer" SectionOne

    ; Set the output path for installation
    SetOutPath $INSTDIR
    
    ; Create shortcuts
    CreateDirectory $DESKTOP
    CreateShortCut "$DESKTOP\fastn.lnk" "cmd.exe $INSTDIR\fastn.exe"
    
    ; Copy application files
    File "fastn.exe"
    ; File "Readme.txt"
    
SectionEnd

; Function to be called when the installer is finished
Function .onInstSuccess
    MessageBox MB_OK "Installation complete!"
FunctionEnd
