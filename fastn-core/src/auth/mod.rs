pub(crate) mod config;
pub(crate) mod github;
pub(crate) mod routes;

mod emailpassword;

mod utils;

use std::str::FromStr;

pub const COOKIE_NAME: &str = "session";

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct FastnUser {
    pub username: String,
    pub id: String,
    pub name: Option<String>,
    pub email: Option<String>,
}

impl FastnUser {
    fn from_row(row: &tokio_postgres::Row) -> Self {
        FastnUser {
            id: row.get("id"),
            username: row.get("username"),
            name: row.get("name"),
            email: row.get("email"),
        }
    }
}

#[derive(Debug)]
pub enum AuthProviders {
    GitHub,
}

impl AuthProviders {
    // pub(crate) const AUTH_ITER: [AuthProviders; 1] = [AuthProviders::GitHub];
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

/// will fetch out the decrypted user data from cookies
/// and return it as string
/// if no cookie wrt to platform found it throws an error
pub async fn get_user_data_from_cookies(
    provider: &str,
    requested_field: &str,
    cookies: &std::collections::HashMap<String, String>,
) -> fastn_core::Result<Option<String>> {
    let session_id = cookies.get(fastn_core::auth::COOKIE_NAME).ok_or_else(|| {
        fastn_core::Error::GenericError("user detail not found in the cookie".to_string())
    });

    match session_id {
        Ok(session_id) => {
            let session_id = uuid::Uuid::from_str(session_id.as_str())?;

            return match fastn_core::auth::AuthProviders::from_str(provider) {
                fastn_core::auth::AuthProviders::GitHub => {
                    // TODO: come back here when user_details is updated to use session id
                    let user: FastnUser =
                        fastn_core::auth::get_authenticated_user(&session_id).await?;

                    match requested_field {
                        "username" | "user_name" | "user-name" => Ok(Some(user.username)),

                        "token" => {
                            return fastn_core::auth::emailpassword::get_token_from_db(
                                &session_id,
                                provider,
                            )
                            .await
                            .map(|t| Some(t.token.to_string()));
                        }

                        _ => Err(fastn_core::Error::GenericError(format!(
                            "invalid field {} requested for platform {}",
                            requested_field, provider
                        ))),
                    }
                }
            };
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

    let session_id =
        cookies
            .get(fastn_core::auth::COOKIE_NAME)
            .ok_or(fastn_core::Error::GenericError(
                "github user detail not found in the cookies".to_string(),
            ));

    match session_id {
        Ok(session_id) => {
            let session_id = uuid::Uuid::from_str(session_id.as_str())?;

            let token = fastn_core::auth::emailpassword::get_token_from_db(&session_id, "github")
                .await
                .map(|t| t.token.to_string())?;

            let user: FastnUser = fastn_core::auth::get_authenticated_user(&session_id).await?;

            let github_ud: github::UserDetail = github::UserDetail {
                access_token: token,
                user,
            };

            matched_identities.extend(github::matched_identities(github_ud, identities).await?);
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
            actix_web::cookie::Cookie::build(fastn_core::auth::COOKIE_NAME, session_id.to_string())
                .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
                .path("/")
                .permanent()
                .finish(),
        )
        // redirect to next
        .append_header((actix_web::http::header::LOCATION, next))
        .finish());
}

pub async fn get_authenticated_user(
    session_id: &uuid::Uuid,
) -> fastn_core::Result<fastn_core::auth::FastnUser> {
    fastn_core::auth::emailpassword::get_user_from_session(session_id).await
}
