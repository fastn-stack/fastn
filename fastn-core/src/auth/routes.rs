use std::str::FromStr;

/// route handler: /-/auth/login/
pub async fn login(
    req: &fastn_core::http::Request,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    if fastn_core::auth::utils::is_authenticated(req) {
        return Ok(fastn_core::http::redirect(next));
    }

    let provider = req.q("provider", "github".to_string())?;

    match provider.as_str() {
        "github" => fastn_core::auth::github::login(req, next).await,
        // client should handle redirects to next for emailpassword login
        "emailpassword" => fastn_core::auth::emailpassword::login(req, next).await,
        _ => Ok(fastn_core::not_found!("unknown provider: {}", provider)),
    }
}

// route: /-/auth/logout/
pub async fn logout(
    req: &fastn_core::http::Request,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    // TODO: Refactor, Not happy with this code, too much of repetition of similar code

    if let Some(session_id) = req.cookie(fastn_core::auth::COOKIE_NAME) {
        let session_id =
            uuid::Uuid::from_str(session_id.as_str()).expect("cookie contains valid uuid");
        let affected = fastn_core::auth::emailpassword::destroy_session(session_id)
            .await
            .unwrap();

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

    match req.path() {
        "/-/auth/login/" => login(&req, next).await,
        // TODO: This has be set while creating the GitHub OAuth Application
        "/-/auth/github/" => fastn_core::auth::github::callback(&req, next).await,
        "/-/auth/logout/" => logout(&req, next).await,

        "/-/auth/create-user/" => fastn_core::auth::emailpassword::create_user(&req).await,

        // "/-/auth/resend-confirmation-email/" => todo!("confirm-email"),
        // "/-/auth/login/?next=/" => todo!(),
        // "/-/auth/send-email-login-code/" => todo!(),
        // "/-/auth/logout/?next=/" => todo!(),
        // "/-/auth/confirm-email/?code=<>" => todo!(),
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
