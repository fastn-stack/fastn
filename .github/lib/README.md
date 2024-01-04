## libpq, libcrypto-3, libssl-3

These DLLs are essential for specific PostgreSQL-related dependencies on Windows.
They are included in the installer and will be extracted into the installation directory of fastn on Windows.

The DLLs are designed for PostgreSQL v16 but are also compatible with version 14. 
However, future compatibility may change, requiring these DLLs to be updated to their latest version.

Note: All these DLLs are x86_64 since the 32-bit versions are not supported by 64-bit systems and binaries (the fastn executable is a 64-bit binary).

Downloaded from:
- [libpq](https://www.dllme.com/dll/files/libpq) / SHA1: 349d82a57355ad6be58cfe983a6b67160892e8cd
- [libssl-3-x64](https://www.dllme.com/dll/files/libssl-3-x64) / SHA1: d57a7a562ffe6bf8c51478da8cd15f9364e97923
- [libcrypto-3-x64](https://www.dllme.com/dll/files/libcrypto-3-x64) / SHA1: 2e2cff1ab2b229cbc0f266bf51a2c08ce06f58e9

These DLLs are included in the install.nsi script in both the installer and uninstall sections.
