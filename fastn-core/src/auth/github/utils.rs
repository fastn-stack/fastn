pub async fn github_client() -> Result<oauth2::basic::BasicClient, fastn_ds::EnvironmentError> {
    let client_id = fastn_ds::DocumentStore::env("FASTN_GITHUB_CLIENT_ID").await?;
    let client_id = oauth2::ClientId::new(client_id);

    let client_secret = fastn_ds::DocumentStore::env("FASTN_GITHUB_CLIENT_SECRET")
        .await
        .ok()
        .map(oauth2::ClientSecret::new);

    Ok(oauth2::basic::BasicClient::new(
        client_id,
        client_secret,
        oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(
            oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                .expect("Invalid token endpoints URL"),
        ),
    ))
}
