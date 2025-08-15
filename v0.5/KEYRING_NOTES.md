# Keyring Usage in fastn

## Overview

The fastn ecosystem uses the system keyring to securely store entity secret keys. This approach provides better security than file-based storage as keys are encrypted by the OS keyring service.

## Keyring Entry Format

Keyring entries are stored as:

- **Service**: `"fastn"`
- **Account/Username**: The entity's ID52 (52-character public key identifier)
- **Password**: The 64-character hex-encoded secret key (stored as string)

```rust
keyring::Entry::new("fastn", id52)
```

## Key Storage Strategy

**Writing**: Use `set_password()` with hex-encoded string
- Better UX: Users can view/copy keys in password managers
- Portable: Hex strings are easy to copy/paste
- Debuggable: Human-readable format

**Reading**: Try both methods for compatibility
1. First try `get_password()` (new format - hex string)
2. Fall back to `get_secret()` (legacy format - raw bytes)
3. This ensures compatibility with existing keys

```rust
// Writing (new format - always use hex)
let entry = keyring::Entry::new("fastn", &id52)?;
entry.set_password(&secret_key.to_string())?;  // hex string

// Reading (support both formats)
let entry = keyring::Entry::new("fastn", &id52)?;
let secret_key = match entry.get_password() {
    Ok(hex_string) => {
        // New format: hex string
        fastn_id52::SecretKey::from_str(&hex_string)?
    }
    Err(_) => {
        // Legacy format: try raw bytes
        let secret_bytes = entry.get_secret()?;
        if secret_bytes.len() != 32 {
            return Err(eyre::anyhow!("Invalid key length"));
        }
        fastn_id52::SecretKey::from_bytes(&secret_bytes[..32])
    }
};
```

## Reading Priority

When reading keys, the order of precedence is:

1. **Environment Variable**: `FASTN_SECRET_KEY` (hex-encoded string)
2. **File**: `.fastn.secret-key` or `entity.private-key` (hex-encoded string) - ONLY if explicitly created by user
3. **Keyring**: Using ID52 from `.fastn.id52` or `entity.id52` file
4. **Error**: If no key found, return error (NO auto-generation)

**Important**: Keys/identities should ONLY be generated when explicitly requested by the user through commands like `fastn-id52 generate`. Never auto-generate keys implicitly.

## File Conventions

- `.fastn.id52` - Contains the public key (ID52 format)
- `.fastn.secret-key` - Contains the secret key (hex format) - ONLY when user explicitly chooses file storage

## Critical Security Rules

1. **NO Automatic Fallback to Disk**: If keyring is unavailable, FAIL with error. Never automatically write secrets to disk
2. **Explicit File Storage**: Writing secrets to files requires explicit user action (e.g., `--file` flag)
3. **Explicit Generation**: Never auto-generate keys without explicit user action
4. **Clear Warnings**: When user chooses file storage, warn about security implications

## fastn-id52 CLI Implementation

The `fastn-id52 generate` command should:

1. **Default behavior** (no flags or `-k`/`--keyring`):
   - Generate new key pair
   - Store secret key in keyring under `keyring::Entry::new("fastn", id52)`
   - Store as hex string using `set_password(&secret_key.to_string())`
   - If keyring fails: ERROR and exit (no fallback to file)
   - Print only the ID52 to stdout
   - Print status message to stderr

2. **With `-f`/`--file [FILENAME]`** (explicit file storage):
   - Generate new key pair
   - Warn user about security implications of file storage
   - Save secret key to file in hex format
   - Print ID52 to stderr
   - Do not use keyring

3. **With `-f -`** (explicit stdout output):
   - Generate new key pair
   - Print secret key (hex) to stdout
   - Print ID52 to stderr
   - Do not store anywhere

## Migration Path

The reading code supports both formats automatically:
- New keys: Stored as hex strings via `set_password()`
- Legacy keys: Stored as raw bytes via `set_secret()`
- Reading code tries `get_password()` first, falls back to `get_secret()`
- No manual migration needed - keys work transparently

## Why Hex Strings?

Storing keys as hex strings in the password field provides better UX:

1. **Password Manager Compatible**: Users can view their keys in password managers
2. **Easy Copy/Paste**: Hex strings can be easily copied and used elsewhere
3. **Debugging**: Developers can verify keys without special tools
4. **Backup**: Users can manually backup keys from their password manager
5. **Cross-platform**: Hex strings work the same everywhere

The 64-character hex string (for 32-byte key) is still secure and fits well within password manager limits.

## Security Considerations

1. **No Implicit Key Generation**: Never generate keys without explicit user request
2. **No Automatic Disk Storage**: Never write secrets to disk without explicit user consent
3. **Clear User Intent**: Commands that generate keys should be clearly named (e.g., `generate`, `create`)
4. **Security Warnings**: When user chooses file storage, display clear warning about risks
5. **Key Visibility**: Secret keys should never be displayed unless explicitly requested with flags like `--file -`

## Error Messages

When keyring is unavailable:
```
Error: Unable to access system keyring for secure key storage.

To proceed, you must explicitly choose an alternative:
  - Use --file to save the secret key to a file (WARNING: less secure)
  - Use --file - to output the key to stdout (you must store it securely yourself)
  - Fix keyring access and retry

Never store secret keys in plain text files unless absolutely necessary.
```