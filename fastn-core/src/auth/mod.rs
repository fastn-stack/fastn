pub(crate) mod config;
pub(crate) mod github;
pub(crate) mod routes;

mod email_password;

mod utils;

pub const COOKIE_NAME: &str = "session";

#[derive(Debug, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable)]
#[diesel(table_name = fastn_core::schema::fastn_user)]
pub struct FastnUser {
    pub id: i32,
    pub username: String,
    #[serde(skip_serializing)]
    pub password: String,
    pub name: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
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
            let session_id: i32 = session_id.parse()?;

            return match fastn_core::auth::AuthProviders::from_str(provider) {
                fastn_core::auth::AuthProviders::GitHub => {
                    let (user, _) =
                        fastn_core::auth::get_authenticated_user_with_email(&session_id).await?;

                    match requested_field {
                        "username" | "user_name" | "user-name" => Ok(Some(user.username)),

                        "token" => {
                            use diesel::prelude::*;
                            use diesel_async::RunQueryDsl;

                            let pool = fastn_core::db::pool().await.as_ref().unwrap();
                            let mut conn =
                                pool.get()
                                    .await
                                    .map_err(|e| fastn_core::Error::DatabaseError {
                                        message: format!("Failed to get connection to db. {:?}", e),
                                    })?;

                            let token = fastn_core::schema::fastn_oauthtoken::table
                                .select(fastn_core::schema::fastn_oauthtoken::token)
                                .filter(
                                    fastn_core::schema::fastn_oauthtoken::session_id
                                        .eq(&session_id),
                                )
                                .filter(fastn_core::schema::fastn_oauthtoken::provider.eq("github"))
                                .first::<String>(&mut conn)
                                .await
                                .optional()
                                .map_err(|e| fastn_core::Error::DatabaseError {
                                    message: format!(
                                        "failed to get token from fastn_oauthtoken: {e}"
                                    ),
                                })?;

                            Ok(token)
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
            use diesel::prelude::*;
            use diesel_async::RunQueryDsl;

            let session_id: i32 = session_id.parse()?;

            let pool = fastn_core::db::pool().await.as_ref().unwrap();
            let mut conn = pool
                .get()
                .await
                .map_err(|e| fastn_core::Error::DatabaseError {
                    message: format!("Failed to get connection to db. {:?}", e),
                })?;

            let token: String = fastn_core::schema::fastn_oauthtoken::table
                .select(fastn_core::schema::fastn_oauthtoken::token)
                .filter(fastn_core::schema::fastn_oauthtoken::session_id.eq(&session_id))
                .filter(fastn_core::schema::fastn_oauthtoken::provider.eq("github"))
                .first::<String>(&mut conn)
                .await
                .map_err(|e| fastn_core::Error::DatabaseError {
                    message: format!("failed to get token from fastn_oauthtoken: {e}"),
                })?;

            let (user, _) =
                fastn_core::auth::get_authenticated_user_with_email(&session_id).await?;

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
    session_id: i32,
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

/// get FastnUser and its primary email from session
pub async fn get_authenticated_user_with_email(
    session_id: &i32,
) -> fastn_core::Result<(fastn_core::auth::FastnUser, String)> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let pool = fastn_core::db::pool().await.as_ref().unwrap();

    let mut conn = pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let user_id: i32 = fastn_core::schema::fastn_session::table
        .select(fastn_core::schema::fastn_session::user_id)
        .filter(fastn_core::schema::fastn_session::id.eq(session_id))
        .first(&mut conn)
        .await?;

    let user: fastn_core::auth::FastnUser = fastn_core::schema::fastn_user::table
        .filter(fastn_core::schema::fastn_user::id.eq(user_id))
        .select(fastn_core::auth::FastnUser::as_select())
        .first(&mut conn)
        .await?;

    let email: fastn_core::utils::CiString = fastn_core::schema::fastn_user_email::table
        .filter(fastn_core::schema::fastn_user_email::user_id.eq(user_id))
        .filter(fastn_core::schema::fastn_user_email::verified.eq(true))
        .filter(fastn_core::schema::fastn_user_email::primary.eq(true))
        .select(fastn_core::schema::fastn_user_email::email)
        .first(&mut conn)
        .await?;

    Ok((user, email.0))
}
