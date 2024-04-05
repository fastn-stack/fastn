pub enum UpdateToken {
    SiteToken(clift::utils::SiteToken),
    GithubToken(clift::utils::GithubOidcActionToken),
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateTokenError {
    #[error("SiteToken: {0}")]
    SiteToken(#[from] std::env::VarError),
    #[error("GithubToken: {0}")]
    GithubToken(#[from] clift::utils::GithubActionIdTokenRequestError),
}

pub fn update_token() -> Result<UpdateToken, UpdateTokenError> {
    match clift::utils::github_oidc_action_token() {
        Ok(token) => Ok(UpdateToken::GithubToken(token)),
        Err(clift::utils::GithubActionIdTokenRequestError::TokenMissing(e)) => {
            eprintln!("Github OIDC Token missing: {e}, trying SiteToken...");
            Ok(UpdateToken::SiteToken(clift::utils::SiteToken::from_env()?))
        }
        Err(e) => Err(e.into()),
    }
}
