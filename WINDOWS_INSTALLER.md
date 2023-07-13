# Fastn Windows Installer

## Introduction

The Windows installer for Fastn is built using NSIS (Nullsoft Scriptable Install System), a popular tool for creating Windows installers. NSIS is configured using its own scripting language. The configuration script is named `install.nsi` and can be found in the root folder. Some changes were made in the `release.yml`, which are mentioned below. Additionally, an icon for the installer named `fastn.ico` was added to the root folder.

## Changes Made

1. Updated the `release.yml` file to incorporate Windows installer support for the Fastn executable.
2. Integrated NSIS into the build process using the `makensis` GitHub Action. This action allows the execution of NSIS scripts during the build workflow.
3. Added the `install.nsi` script to the root folder of the Fastn project. This script configures the NSIS installer.
4. Some other important details:
    - The installer uses the NSIS MUI.
    - The color scheme is set to a dark color scheme to match the color scheme of the Fastn website:
      ```nsi
      !define MUI_INSTFILESPAGE_COLORS "FFFFFF 000000"
      !define MUI_BGCOLOR 000000
      !define MUI_TEXTCOLOR ffffff
      ```
    - The default icon is replaced with `fastn.ico`.
    ```nsi
    !define MUI_ICON "fastn.ico"
    ```
    - We are using version 3 of NSIS.

## Installer Functionality

The Fastn installer performs the following tasks:

1. Shows a Welcome and License Page.
2. Extracts all necessary files to either the default location (Program Files) or a user-defined folder.
3. Checks if the required path variable is already set up on the system. If not, it automatically configures the correct path variable to ensure seamless execution of Fastn without any issues.

## Code Changes

The following code in the `release-windows` job is responsible for building the installer from the executable built by `cargo` in the previous step:

```yaml
- name: Download EnVar Plugin for NSIS
  uses: carlosperate/download-file-action@v1.0.3
  with:
    file-url: https://nsis.sourceforge.io/mediawiki/images/7/7f/EnVar_plugin.zip
    file-name: envar_plugin.zip
    location: ${{ github.workspace }}
- name: Extract EnVar plugin
  run: 7z x -o"${{ github.workspace }}/NSIS_Plugins" "${{ github.workspace }}/envar_plugin.zip"
- name: Create installer
  uses: joncloud/makensis-action@v4
  with:
    arguments: /V3 /DCURRENT_WD=${{ github.workspace }} /DVERSION=${{ github.event.inputs.releaseTag }}
    additional-plugin-paths: ${{ github.workspace }}/NSIS_Plugins/Plugins
- uses: actions/upload-artifact@v2
  with:
    name: windows_x64_installer.exe
    path: windows_x64_installer.exe
```

Explanation:

1. Download the EnVar Plugin for NSIS, which is required for correctly configuring path variables in Windows.
2. Extract the plugin to the appropriate location.
3. Create the installer executable by specifying the following inputs:
   - `CURRENT_WD`: The current Github Working Directory.
   - `VERSION`: The release tag.

In the `create-release` job, we download the `windows_x64_installer.exe` artifact and rename it to `fastn_setup`. In the next step, it is added to the `artifacts` list as part of the files to be released.