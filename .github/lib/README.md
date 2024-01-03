## libpq, libcrypto-3, libssl-3

These DLLs are required for certain PostgreSQL-related dependencies on Windows. They are being packaged in the installer and will be extracted in the fastn's installation directory on Windows.

The DLLs are designed for PostgreSQL v16 but are also compatible with version 14. However, in the future, this compatibility may change, and these DLLs will need to be updated to their latest version.

Downloaded from:
- [libpq](https://www.dllme.com/dll/files/libpq)
- [libssl-3-x64](https://www.dllme.com/dll/files/libssl-3-x64)
- [libcrypto-3-x64](https://www.dllme.com/dll/files/libcrypto-3-x64)

These DLLs are included in the install.nsi script in both the installer and uninstall sections.
