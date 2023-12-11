pub(crate) mod config;
pub(crate) mod github;
pub(crate) mod routes;

mod emailpassword;

mod utils;

pub use utils::{decrypt, encrypt};

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FastnUser {
    pub username: String,
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

#[derive(Debug)]
pub(crate) enum AuthProviders {
    GitHub,
}

impl AuthProviders {
    pub(crate) const AUTH_ITER: [AuthProviders; 1] = [AuthProviders::GitHub];
    pub(crate) fn as_str(&self) -> &'static str {
        match self {
            AuthProviders::GitHub => "github",
        }
    }

    pub(crate) fn from_str(s: &str) -> Self {
        match s {
            "github" => AuthProviders::GitHub,
            _ => panic!("Invalid auth provider {}", s),
        }
    }
}

pub fn secret_key() -> String {
    match std::env::var("FASTN_SECRET_KEY") {
        Ok(secret) => secret,
        Err(_e) => {
            println!("WARN: SECRET_KEY not set");
            // TODO: Need to change this approach later
            "FASTN_TEMP_SECRET".to_string()
        }
    }
}

/// will fetch out the decrypted user data from cookies
/// and return it as string
/// if no cookie wrt to platform found it throws an error
pub async fn get_user_data_from_cookies(
    platform: &str,
    requested_field: &str,
    cookies: &std::collections::HashMap<String, String>,
) -> fastn_core::Result<Option<String>> {
    let ud_encrypted = cookies.get(platform).ok_or_else(|| {
        fastn_core::Error::GenericError(format!(
            "user detail not found for platform {} in the cookies",
            platform
        ))
    });
    match ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(ud_decrypted) = utils::decrypt(encrypt_str).await {
                return match fastn_core::auth::AuthProviders::from_str(platform) {
                    fastn_core::auth::AuthProviders::GitHub => {
                        let github_ud: github::UserDetail =
                            serde_json::from_str(ud_decrypted.as_str())?;
                        match requested_field {
                            "username" | "user_name" | "user-name" => {
                                Ok(Some(github_ud.user.username))
                            }
                            "token" => Ok(Some(github_ud.access_token)),
                            _ => Err(fastn_core::Error::GenericError(format!(
                                "invalid field {} requested for platform {}",
                                requested_field, platform
                            ))),
                        }
                    }
                };
            }
        }
        Err(err) => {
            // Debug out the error and return None
            let error_msg = format!("User data error: {}", err);
            dbg!(&error_msg);
        }
    };
    Ok(None)
}

pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    let mut matched_identities: Vec<fastn_core::user_group::UserIdentity> = vec![];

    let github_ud_encrypted = cookies
        .get(fastn_core::auth::AuthProviders::GitHub.as_str())
        .ok_or_else(|| {
            fastn_core::Error::GenericError(
                "github user detail not found in the cookies".to_string(),
            )
        });
    match github_ud_encrypted {
        Ok(encrypt_str) => {
            if let Ok(github_ud_decrypted) = utils::decrypt(encrypt_str).await {
                let github_ud: github::UserDetail =
                    serde_json::from_str(github_ud_decrypted.as_str())?;
                matched_identities.extend(github::matched_identities(github_ud, identities).await?);
            }
        }
        Err(err) => {
            // TODO: What to do with this error
            format!("{}{}", "github user detail not found in the cookies", err);
        }
    };
    Ok(matched_identities)
}
