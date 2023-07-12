; !define BINARY_PATH

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

    ; Set the output path for installation
    SetOutPath $INSTDIR
    
    ; Copy application files
    File "${BINARY_PATH}"
    ; File "Readme.txt"
    
SectionEnd

; Function to be called when the installer is finished
Function .onInstSuccess
    MessageBox MB_OK "Installation complete!"
FunctionEnd