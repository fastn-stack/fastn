/// route handler: /-/auth/login/
pub async fn login(
    req: &fastn_core::http::Request,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    if fastn_core::auth::utils::is_authenticated(req) {
        return Ok(fastn_core::http::redirect(next));
    }

    let provider = req.q("provider", "github".to_string())?;

    match provider.as_str() {
        "github" => fastn_core::auth::github::login(req, next).await,
        // client should handle redirects to next for email_password login
        "emailpassword" => fastn_core::auth::email_password::login(req, db_pool, next).await,
        _ => Ok(fastn_core::not_found!("unknown provider: {}", provider)),
    }
}

// route: /-/auth/logout/
pub async fn logout(
    req: &fastn_core::http::Request,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    if let Some(session_id) = req.cookie(fastn_core::auth::COOKIE_NAME) {
        let session_id: i32 = session_id.parse()?;

        let mut conn = db_pool
            .get()
            .await
            .map_err(|e| fastn_core::Error::DatabaseError {
                message: format!("Failed to get connection to db. {:?}", e),
            })?;

        let affected = diesel::delete(fastn_core::schema::fastn_session::table)
            .filter(fastn_core::schema::fastn_session::id.eq(&session_id))
            .execute(&mut conn)
            .await?;

        tracing::info!("session destroyed for {session_id}. Rows affected {affected}.");
    }

    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::COOKIE_NAME, "")
                .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, next))
        .finish())
}

// handle: if request.url starts with /-/auth/
#[tracing::instrument(skip_all)]
pub async fn handle_auth(
    req: fastn_core::http::Request,
) -> fastn_core::Result<fastn_core::http::Response> {
    let next = req.q("next", "/".to_string())?;

    let pool =
        fastn_core::db::pool()
            .await
            .as_ref()
            .map_err(|e| fastn_core::Error::DatabaseError {
                message: format!("Failed to get connection to db. {:?}", e),
            })?;

    match req.path() {
        "/-/auth/login/" => login(&req, pool, next).await,
        // TODO: This has be set while creating the GitHub OAuth Application
        "/-/auth/github/" => fastn_core::auth::github::callback(&req, pool, next).await,
        "/-/auth/logout/" => logout(&req, pool, next).await,

        "/-/auth/create-user/" => {
            fastn_core::auth::email_password::create_user(&req, pool, next).await
        }
        "/-/auth/confirm-email/" => {
            fastn_core::auth::email_password::confirm_email(&req, pool).await
        }
        "/-/auth/resend-confirmation-email/" => {
            fastn_core::auth::email_password::resend_email(&req, pool).await
        }

        // "/-/auth/send-email-login-code/" => todo!(),
        // "/-/auth/add-email/" => todo!(),
        // "/-/auth/update-name/" => todo!(),
        // "/-/auth/update-password/" => todo!(),
        // "/-/auth/update-username/" => todo!(),
        // "/-/auth/update-email/" => todo!(),
        // "/-/auth/disable-account/" => todo!(),
        // "/-/auth/close-sessions/?session=<session-id|all>" => todo!(),
        _ => Ok(fastn_core::not_found!("route not found: {}", req.path())),
    }
}
