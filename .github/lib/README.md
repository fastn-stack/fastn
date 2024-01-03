## libpq, libcrypto-3, libssl-3

These DLLs are required for some PostgreSQL related dependencies on Windows. these are being packaged in the the installer and will be extracted in the fastn's installation directory on Windows.

The DLLs are for PostgreSQL v16 but can also work with version 14. But in future this may change then these DLLs will have to be updated to their latest version.

Downloaded from:
- https://www.dllme.com/dll/files/libpq
- https://www.dllme.com/dll/files/libssl-3-x64
- https://www.dllme.com/dll/files/libcrypto-3-x64

These DLLs are included in the install.nsi script in the installer and uninstall sections.
