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

async fn set_session_cookie_and_end_response(
    req: &fastn_core::http::Request,
    session_id: uuid::Uuid,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    return Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build("session", session_id.to_string())
                .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
                .path("/")
                .permanent()
                .finish(),
        )
        // redirect to next
        .append_header((actix_web::http::header::LOCATION, next))
        .finish());
}

async fn insert_oauth_token(
    session_id: uuid::Uuid,
    token: &str,
    provider: AuthProviders,
) -> fastn_core::Result<u64> {
    let client = fastn_core::auth::emailpassword::db::get_client().await?;

    let id = uuid::Uuid::new_v4();

    Ok(client
        .execute(
            "insert into fastn_oauthtoken(id, session_id, token, provider) values ($1, $2, $3, $4)",
            &[&id, &session_id, &token, &provider.as_str()],
        )
        .await
        .unwrap())
}
