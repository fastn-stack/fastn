// Lazy means a value which initialize at the first time access
// we have to access it before using it and make sure to use it while starting a server
// TODO: they should be configured with auth feature flag
// if feature flag auth is enabled Make sure that before accessing in the API these variable
// are set
static GITHUB_CLIENT_ID: once_cell::sync::Lazy<oauth2::ClientId> = {
    once_cell::sync::Lazy::new(|| {
        oauth2::ClientId::new(match std::env::var("FASTN_GITHUB_CLIENT_ID") {
            Ok(val) => val,
            Err(e) => format!("{}{}", "FASTN_GITHUB_CLIENT_ID not set in env ", e),
        })
    })
};

static GITHUB_CLIENT_SECRET: once_cell::sync::Lazy<oauth2::ClientSecret> = {
    once_cell::sync::Lazy::new(|| {
        oauth2::ClientSecret::new(match std::env::var("FASTN_GITHUB_CLIENT_SECRET") {
            Ok(val) => val,
            Err(e) => format!("{}{}", "FASTN_GITHUB_CLIENT_SECRET not set in env ", e),
        })
    })
};

pub fn github_client() -> oauth2::basic::BasicClient {
    oauth2::basic::BasicClient::new(
        GITHUB_CLIENT_ID.to_owned(),
        Some(GITHUB_CLIENT_SECRET.to_owned()),
        oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
        Some(
            oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                .expect("Invalid token endpoint URL"),
        ),
    )
}
