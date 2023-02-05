use magic_crypt::MagicCryptError;

// 127.0.0.1:8000 -> 127.0.0.1
pub fn domain(host: &str) -> String {
    match host.split_once(':') {
        Some((domain, _port)) => domain.to_string(),
        None => host.to_string(),
    }
}
pub async fn get_api<T: serde::de::DeserializeOwned>(url: &str, token: &str) -> fastn_core::Result<T> {
    let response = reqwest::Client::new()
        .get(url)
        .header(reqwest::header::AUTHORIZATION, token)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("fastn"),
        )
        .send()
        .await?;

    if !response.status().eq(&reqwest::StatusCode::OK) {
        return Err(fastn_core::Error::APIResponseError(format!(
            "fastn-API-ERROR: {}, Error: {}",
            url,
            response.text().await?
        )));
    }

    Ok(response.json().await?)
}
pub async fn encrypt_str(user_detail_str: &String) -> String {
    use magic_crypt::MagicCryptTrait;
    let secret_key = fastn_core::auth::secret_key();
    let mc_obj = magic_crypt::new_magic_crypt!(secret_key.as_str(), 256);
    mc_obj
        .encrypt_to_base64(user_detail_str)
        .as_str()
        .to_owned()
}
pub async fn decrypt_str(encrypted_str: &String) -> Result<String, MagicCryptError> {
    use magic_crypt::MagicCryptTrait;
    let secret_key = fastn_core::auth::secret_key();
    let mc_obj = magic_crypt::new_magic_crypt!(&secret_key, 256);
    mc_obj.decrypt_base64_to_string(encrypted_str)
}
pub fn is_login(req: &actix_web::HttpRequest) -> bool {
    let mut found_cookie = false;
    for auth_provider in fastn_core::auth::AuthProviders::AUTH_ITER.iter() {
        dbg!(&auth_provider);
        if req.cookie(auth_provider.as_str()).is_some() {
            found_cookie = true;
        }
    }
    found_cookie
}
