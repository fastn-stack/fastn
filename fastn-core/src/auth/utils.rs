// 127.0.0.1:8000 -> 127.0.0.1
pub fn domain(host: &str) -> String {
    match host.split_once(':') {
        Some((domain, _port)) => domain.to_string(),
        None => host.to_string(),
    }
}

pub async fn encrypt(input: &str) -> String {
    use magic_crypt::MagicCryptTrait;
    let secret_key = fastn_core::auth::utils::secret_key().await;
    let mc_obj = magic_crypt::new_magic_crypt!(secret_key.as_str(), 256);
    mc_obj.encrypt_to_base64(input).as_str().to_owned()
}

pub async fn decrypt(input: &str) -> Result<String, magic_crypt::MagicCryptError> {
    use magic_crypt::MagicCryptTrait;
    let secret_key = fastn_core::auth::utils::secret_key().await;
    let mc_obj = magic_crypt::new_magic_crypt!(&secret_key, 256);
    mc_obj.decrypt_base64_to_string(input)
}

pub async fn secret_key() -> String {
    match fastn_ds::DocumentStore::env("FASTN_SECRET_KEY").await {
        Ok(secret) => secret,
        Err(_e) => {
            fastn_core::warning!(
                "WARN: Using default SECRET_KEY. Provide one using FASTN_SECRET_KEY env var."
            );
            "FASTN_TEMP_SECRET".to_string()
        }
    }
}

pub fn is_authenticated(req: &fastn_core::http::Request) -> bool {
    req.cookie(fastn_core::auth::SESSION_COOKIE_NAME).is_some()
}
