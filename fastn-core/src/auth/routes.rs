// route: /-/auth/login/
pub async fn login(req: &fastn_core::http::Request) -> fastn_core::Result<actix_web::HttpResponse> {
    if fastn_core::auth::utils::is_authenticated(req) {
        return Ok(fastn_core::http::redirect("/".to_string()));
    }

    let platform = req.q("platform", "github".to_string())?;

    match platform.as_str() {
        "github" => fastn_core::auth::github::login(req).await,
        _ => {
            return Ok(actix_web::HttpResponse::BadRequest()
                .body("Please select the platform, by which you want to login"));
        } // _ => unreachable!(),
    }
}

// route: /-/auth/logout/
pub fn logout() -> fastn_core::Result<actix_web::HttpResponse> {
    // TODO: Refactor, Not happy with this code, too much of repetition of similar code
    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::GitHub.as_str(), "")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish())
}

// handle: if request.url starts with /-/auth/
#[tracing::instrument(skip_all)]
pub async fn handle_auth(
    req: fastn_core::http::Request,
) -> fastn_core::Result<fastn_core::http::Response> {
    match req.path() {
        "/-/auth/login/" => login(&req).await,
        // TODO: This has be set while creating the GitHub OAuth Application
        "/-/auth/github/" => fastn_core::auth::github::callback(&req).await,
        "/-/auth/logout/" => logout(),
        _ => Ok(fastn_core::not_found!("route not found: {}", req.path())),
    }
}
