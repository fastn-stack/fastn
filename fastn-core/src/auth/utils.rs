use magic_crypt::MagicCryptError;

// 127.0.0.1:8000 -> 127.0.0.1
pub fn domain(host: &str) -> String {
    match host.split_once(':') {
        Some((domain, _port)) => domain.to_string(),
        None => host.to_string(),
    }
}

pub async fn encrypt(user_detail_str: &String) -> String {
    use magic_crypt::MagicCryptTrait;
    let secret_key = fastn_core::auth::secret_key();
    let mc_obj = magic_crypt::new_magic_crypt!(secret_key.as_str(), 256);
    mc_obj
        .encrypt_to_base64(user_detail_str)
        .as_str()
        .to_owned()
}

pub async fn decrypt(encrypted_str: &String) -> Result<String, MagicCryptError> {
    use magic_crypt::MagicCryptTrait;
    let secret_key = fastn_core::auth::secret_key();
    let mc_obj = magic_crypt::new_magic_crypt!(&secret_key, 256);
    if encrypted_str.starts_with('"') {
        // django adds quotes to the cookie value
        dbg!(mc_obj.decrypt_base64_to_string(&encrypted_str[1..encrypted_str.len() - 1]))
    } else {
        dbg!(mc_obj.decrypt_base64_to_string(encrypted_str))
    }
}

pub fn is_authenticated(req: &fastn_core::http::Request) -> bool {
    let mut found_cookie = false;
    for auth_provider in fastn_core::auth::AuthProviders::AUTH_ITER.iter() {
        dbg!(&auth_provider);
        if req.cookie(auth_provider.as_str()).is_some() {
            found_cookie = true;
        }
    }
    found_cookie
}
