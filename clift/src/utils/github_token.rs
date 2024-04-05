pub struct GithubOidcActionToken {
    pub token: String,
    pub url: String,
}

#[derive(Debug, thiserror::Error)]
pub enum GithubActionIdTokenRequestError {
    #[error("Token missing {0}")]
    TokenMissing(std::env::VarError),
    #[error("Url missing {0}")]
    UrlMissing(std::env::VarError),
}

pub fn github_oidc_action_token() -> Result<GithubOidcActionToken, GithubActionIdTokenRequestError>
{
    let token = std::env::var("ACTIONS_ID_TOKEN_REQUEST_TOKEN")
        .map_err(GithubActionIdTokenRequestError::TokenMissing)?;
    let url = std::env::var("ACTIONS_ID_TOKEN_REQUEST_URL")
        .map_err(GithubActionIdTokenRequestError::UrlMissing)?;

    Ok(GithubOidcActionToken { token, url })
}
