// route: /-/auth/login/
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
        _ => Ok(fastn_core::not_found!("unknown provider: {}", provider)),
    }
}

// route: /-/auth/logout/
pub fn logout(
    req: &fastn_core::http::Request,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    // TODO: Refactor, Not happy with this code, too much of repetition of similar code
    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::GitHub.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build("github_user", "")
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
        "/-/auth/logout/" => logout(&req, next),
        _ => Ok(fastn_core::not_found!("route not found: {}", req.path())),
    }
}
