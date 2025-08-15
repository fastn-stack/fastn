use eyre::WrapErr;

/// Environment variable name for providing secret key.
pub const SECRET_KEY_ENV_VAR: &str = "FASTN_SECRET_KEY";

/// Default file name for storing secret key.
pub const SECRET_KEY_FILE: &str = ".fastn.secret-key";

/// Default file name for storing ID52 public key.
pub const ID52_FILE: &str = ".fastn.id52";

/// Generates a new Ed25519 secret key and returns its ID52.
///
/// # Returns
///
/// A tuple of (ID52 string, SecretKey).
pub fn generate_secret_key() -> eyre::Result<(String, fastn_id52::SecretKey)> {
    let secret_key = fastn_id52::SecretKey::generate();
    let id52 = secret_key.id52();
    Ok((id52, secret_key))
}

/// Generates a new secret key and saves it to the system keyring.
///
/// The key is stored in the system keyring under the ID52 identifier,
/// and the ID52 is written to `.fastn.id52` file.
///
/// # Errors
///
/// Returns an error if keyring access or file write fails.
pub async fn generate_and_save_key() -> eyre::Result<(String, fastn_id52::SecretKey)> {
    let (id52, secret_key) = generate_secret_key()?;
    let e = keyring_entry(&id52)?;
    e.set_secret(&secret_key.to_bytes())
        .wrap_err_with(|| format!("failed to save secret key for {id52}"))?;
    tokio::fs::write(ID52_FILE, &id52).await?;
    Ok((id52, secret_key))
}

fn keyring_entry(id52: &str) -> eyre::Result<keyring::Entry> {
    keyring::Entry::new("fastn", id52)
        .wrap_err_with(|| format!("failed to create keyring Entry for {id52}"))
}

fn handle_secret(secret: &str) -> eyre::Result<(String, fastn_id52::SecretKey)> {
    use std::str::FromStr;
    let secret_key = fastn_id52::SecretKey::from_str(secret).map_err(|e| eyre::anyhow!("{}", e))?;
    let id52 = secret_key.id52();
    Ok((id52, secret_key))
}

/// Gets a secret key for a given ID52 and path.
///
/// **Note**: Currently unimplemented, will be implemented in future versions.
///
/// # Panics
///
/// Always panics with "implement for fastn".
pub fn get_secret_key(_id52: &str, _path: &str) -> eyre::Result<fastn_id52::SecretKey> {
    // intentionally left unimplemented as design is changing in fastn
    // this is not used in fastn
    todo!("implement for fastn")
}

/// Reads an existing secret key or creates a new one if none exists.
///
/// Attempts to read the secret key in the following order:
/// 1. From `FASTN_SECRET_KEY` environment variable
/// 2. From `.fastn.secret-key` file
/// 3. From system keyring using ID52 from `.fastn.id52` file
/// 4. Generates new key if none found
///
/// # Errors
///
/// Returns an error if key reading fails (but not if key doesn't exist).
#[tracing::instrument]
pub async fn read_or_create_key() -> eyre::Result<(String, fastn_id52::SecretKey)> {
    if let Ok(secret) = std::env::var(SECRET_KEY_ENV_VAR) {
        tracing::info!("Using secret key from environment variable {SECRET_KEY_ENV_VAR}");
        return handle_secret(&secret);
    } else {
        match tokio::fs::read_to_string(SECRET_KEY_FILE).await {
            Ok(secret) => {
                tracing::info!("Using secret key from file {SECRET_KEY_FILE}");
                let secret = secret.trim_end();
                return handle_secret(secret);
            }
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => {}
            Err(e) => {
                tracing::error!("failed to read {SECRET_KEY_FILE}: {e}");
                return Err(e.into());
            }
        }
    }

    tracing::info!("No secret key found in environment or file, trying {ID52_FILE}");
    match tokio::fs::read_to_string(ID52_FILE).await {
        Ok(id52) => {
            let e = keyring_entry(&id52)?;
            match e.get_secret() {
                Ok(secret) => {
                    if secret.len() != 32 {
                        return Err(eyre::anyhow!(
                            "keyring: secret for {id52} has invalid length: {}",
                            secret.len()
                        ));
                    }

                    let bytes: [u8; 32] = secret.try_into().expect("already checked for length");
                    let secret_key = fastn_id52::SecretKey::from_bytes(&bytes);
                    let id52 = secret_key.id52();
                    Ok((id52, secret_key))
                }
                Err(e) => {
                    tracing::error!("failed to read secret for {id52} from keyring: {e}");
                    Err(e.into())
                }
            }
        }
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => generate_and_save_key().await,
        Err(e) => {
            tracing::error!("failed to read {ID52_FILE}: {e}");
            Err(e.into())
        }
    }
}
