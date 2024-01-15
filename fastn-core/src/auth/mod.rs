pub(crate) mod config;
pub(crate) mod github;
pub(crate) mod routes;
pub(crate) mod utils;

mod email_password;

pub const COOKIE_NAME: &str = "fastn_session";

#[derive(
    Debug, PartialEq, serde::Deserialize, serde::Serialize, diesel::Queryable, diesel::Selectable,
)]
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
                    let user = match fastn_core::auth::get_authenticated_user_with_email(
                        &session_id,
                    )
                    .await
                    {
                        Err(e) => {
                            tracing::error!(
                                "couldn't retrieve authenticated user. Reason: {:?}",
                                e
                            );

                            if e == AuthUserError::UserDoesNotExist {
                                return Err(fastn_core::Error::GenericError(
                                    "User does not exist".to_string(),
                                ));
                            } else if let AuthUserError::UserExistsWithUnverifiedEmail(_) = e {
                                return Err(fastn_core::Error::GenericError(
                                    "User is not verified".to_string(),
                                ));
                            }

                            return Err(fastn_core::Error::GenericError(
                                "Failed to query database".to_string(),
                            ));
                        }

                        Ok((user, _)) => user,
                    };

                    match requested_field {
                        "username" | "user_name" | "user-name" => Ok(Some(user.username)),

                        "token" => {
                            use diesel::prelude::*;
                            use diesel_async::RunQueryDsl;

                            let pool = fastn_core::db::pool().await.as_ref().map_err(|e| {
                                fastn_core::Error::DatabaseError {
                                    message: format!("Failed to get connection to db. {:?}", e),
                                }
                            })?;

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

            let pool = fastn_core::db::pool().await.as_ref().map_err(|e| {
                fastn_core::Error::DatabaseError {
                    message: format!("Failed to get connection to db. {:?}", e),
                }
            })?;

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

            let user = match fastn_core::auth::get_authenticated_user_with_email(&session_id).await
            {
                Err(e) => {
                    tracing::error!("couldn't retrieve authenticated user. Reason: {:?}", e);

                    if e == AuthUserError::UserDoesNotExist {
                        return Err(fastn_core::Error::GenericError(
                            "User does not exist".to_string(),
                        ));
                    } else if let AuthUserError::UserExistsWithUnverifiedEmail(_) = e {
                        return Err(fastn_core::Error::GenericError(
                            "User is not verified".to_string(),
                        ));
                    }

                    return Err(fastn_core::Error::GenericError(
                        "Failed to query database".to_string(),
                    ));
                }

                Ok((user, _)) => user,
            };

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

async fn set_session_cookie_and_redirect_to_next(
    req: &fastn_core::http::Request,
    session_id: i32,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    let (user, email) = match fastn_core::auth::get_authenticated_user_with_email(&session_id).await
    {
        Err(e) => {
            tracing::error!("couldn't retrieve authenticated user. Reason: {:?}", e);

            if e == AuthUserError::UserDoesNotExist {
                return Err(fastn_core::Error::GenericError(
                    "User does not exist".to_string(),
                ));
            } else if let AuthUserError::UserExistsWithUnverifiedEmail(_) = e {
                return fastn_core::http::user_err(
                    // TODO: there should be an option to configure the resend verification
                    // mail webpage
                    vec![("username", "User is not verified")],
                    fastn_core::http::StatusCode::BAD_REQUEST,
                );
            }

            return Err(fastn_core::Error::GenericError(
                "Failed to query database".to_string(),
            ));
        }

        Ok(data) => data,
    };

    let cookie_json = serde_json::json!({
        "session_id": session_id,
        "user": {
            "username": user.username,
            "name": user.name,
            "email": email,
        }
    });

    let encrypted_cookie = fastn_core::auth::utils::encrypt(&cookie_json.to_string()).await;

    return Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::COOKIE_NAME, encrypted_cookie)
                .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
                .path("/")
                .permanent()
                .finish(),
        )
        // redirect to next
        .append_header((actix_web::http::header::LOCATION, next))
        .finish());
}

#[derive(PartialEq, thiserror::Error, Debug)]
pub enum AuthUserError {
    #[error("User exists but doesn't have a verified email")]
    UserExistsWithUnverifiedEmail(fastn_core::auth::FastnUser),

    #[error("User does not exist")]
    UserDoesNotExist,

    #[error("Failed to query db. Details: {0:?}")]
    WrongQuery(#[from] diesel::result::Error),

    #[error("Faile to get a connection to the database, reason: {reason:?}")]
    Connection { reason: String },
}

/// get FastnUser and its primary email from session
pub async fn get_authenticated_user_with_email(
    session_id: &i32,
) -> Result<(fastn_core::auth::FastnUser, String), AuthUserError> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let pool = fastn_core::db::pool()
        .await
        .as_ref()
        .map_err(|e| AuthUserError::Connection {
            reason: format!("{:?}", e),
        })?;

    let mut conn = pool.get().await.map_err(|e| AuthUserError::Connection {
        reason: format!("{:?}", e),
    })?;

    let user_id: Option<i32> = fastn_core::schema::fastn_session::table
        .select(fastn_core::schema::fastn_session::user_id)
        .filter(fastn_core::schema::fastn_session::id.eq(session_id))
        .first(&mut conn)
        .await
        .optional()?;

    if user_id.is_none() {
        return Err(AuthUserError::UserDoesNotExist);
    }

    let user_id = user_id.expect("user_id must be Some");

    let user: Option<fastn_core::auth::FastnUser> = fastn_core::schema::fastn_user::table
        .filter(fastn_core::schema::fastn_user::id.eq(user_id))
        .select(fastn_core::auth::FastnUser::as_select())
        .first(&mut conn)
        .await
        .optional()?;

    if user.is_none() {
        return Err(AuthUserError::UserDoesNotExist);
    }

    let user = user.expect("user must be Some");

    let email: Option<fastn_core::utils::CiString> = fastn_core::schema::fastn_user_email::table
        .filter(fastn_core::schema::fastn_user_email::user_id.eq(user_id))
        .filter(fastn_core::schema::fastn_user_email::verified.eq(true))
        .filter(fastn_core::schema::fastn_user_email::primary.eq(true))
        .select(fastn_core::schema::fastn_user_email::email)
        .first(&mut conn)
        .await
        .optional()?;

    if email.is_none() {
        return Err(AuthUserError::UserExistsWithUnverifiedEmail(user));
    }

    let email = email.expect("email must be Some");

    Ok((user, email.0))
}

pub(crate) fn enable_auth() -> fastn_core::Result<()> {
    use diesel::Connection;
    use diesel_migrations::MigrationHarness;

    let db_url = std::env::var("FASTN_DB_URL")?;

    const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
        diesel_migrations::embed_migrations!();

    let mut conn = diesel::pg::PgConnection::establish(&db_url).map_err(|e| {
        fastn_core::Error::DatabaseError {
            message: format!("Failed to connect to db. {:?}", e),
        }
    })?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to run migrations. {:?}", e),
        })?;

    Ok(())
}
