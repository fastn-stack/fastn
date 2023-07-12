# Fastn Windows Installer Support

This pull request adds Windows installer support to Fastn.

## Introduction

This pull request aims to introduce a Windows installer for fastn. It includes the necessary changes to the `release.yml` file and the addition of an NSIS (Nullsoft Scriptable Install System) configuration script called `install.nsi`. By leveraging NSIS, a popular tool for creating Windows installers, we simplify the deployment and setup process for Fastn on Windows platforms.

## Changes Made

The following changes have been implemented in this pull request:

1. Updated the `release.yml` file to incorporate Windows installer support for the Fastn executable.
2. Integrated NSIS into the build process using the `makensis` GitHub Action. This action allows the execution of NSIS scripts during the build workflow.
3. Added the `install.nsi` script to the root folder of the Fastn project. This script configures the NSIS installer.
4. The installer currently utilizes the Standard UI provided by NSIS. However, due to limitations with the GitHub Action, the Modern UI (MUI) of NSIS will be implemented in a future update to enhance the user experience and improve the visual appeal of the installation process.

## Installer Functionality

The Fastn installer performs the following tasks:

1. Extracts all necessary files to either the default location (Program Files) or a user-defined folder.
2. Checks if the required path variable is already set up on the system. If not, it automatically configures the correct path variable to ensure seamless execution of Fastn without any issues.

## Code Changes

The following code changes have been made to the `release windows` job:

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
    arguments: /DBINARY_PATH=${{ github.workspace }}\target\release\fastn.exe /DVERSION=${{ github.event.inputs.releaseTag }}
    additional-plugin-paths: ${{ github.workspace }}/NSIS_Plugins/Plugins
- uses: actions/upload-artifact@v2
  with:
    name: windows_x64_installer.exe
    path: windows_x64_installer.exe
```

These code changes have been added to the `release windows` job in the workflow file. The steps include:

1. Downloading the EnVar Plugin for NSIS, which is required for correctly configuring path variables in Windows.
2. Extracting the plugin to the appropriate location.
3. Creating the installer executable by specifying the following inputs:
   - `BINARY_PATH`: The absolute path of the Fastn executable generated using Cargo.
   - `VERSION`: The release tag.