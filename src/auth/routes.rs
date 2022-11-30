pub fn is_login(req: &actix_web::HttpRequest) -> bool {
    req.cookie(fpm::auth::AuthProviders::GitHub.as_str())
        .is_some()
        || req
            .cookie(fpm::auth::AuthProviders::TeleGram.as_str())
            .is_some()
        || req
            .cookie(fpm::auth::AuthProviders::Discord.as_str())
            .is_some()
        || req
            .cookie(fpm::auth::AuthProviders::Slack.as_str())
            .is_some()
        || req
            .cookie(fpm::auth::AuthProviders::Google.as_str())
            .is_some()
}

// route: /auth/login/
pub async fn login(
    req: actix_web::HttpRequest,
    edition: Option<String>,
) -> fpm::Result<actix_web::HttpResponse> {
    if is_login(&req) {
        return Ok(actix_web::HttpResponse::Found()
            .append_header((actix_web::http::header::LOCATION, "/".to_string()))
            .finish());
    }

    #[derive(serde::Deserialize)]
    pub struct QueryParams {
        pub platform: String,
    }
    let query = match actix_web::web::Query::<QueryParams>::from_query(req.query_string()) {
        Ok(q) => q,
        Err(err) => {
            dbg!(err);
            return Ok(actix_web::HttpResponse::BadRequest()
                .body("Please select the platform, by which you want to login"));
        }
    };
    match query.platform.as_str() {
        "github" => fpm::auth::github::login(req).await,
        "telegram" => fpm::auth::telegram::login(req).await,
        // TODO: Remove this after demo
        _ => {
            let mut req = fpm::http::Request::from_actix(req, actix_web::web::Bytes::new());
            req.path = "/sorry/".to_string();
            fpm::commands::serve::serve(req, edition).await
        }
        // "discord" => unreachable!(),
        // _ => unreachable!(),
    }
}

// route: /auth/logout/
pub fn logout(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    // TODO: Refactor, Not happy with this code, too much of repetition of similar code
    // It is logging out from all the platforms

    // Ideally it should capture the platform in the request and then logged out
    // only from that platform

    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::AuthProviders::GitHub.as_str(), "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::AuthProviders::TeleGram.as_str(), "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::AuthProviders::Slack.as_str(), "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::AuthProviders::Discord.as_str(), "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fpm::auth::AuthProviders::Google.as_str(), "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish())
}

// handle: if request.url starts with /auth/
pub async fn handle_auth(
    req: actix_web::HttpRequest,
    edition: Option<String>,
) -> fpm::Result<fpm::http::Response> {
    match req.path() {
        "/auth/login/" => login(req, edition).await,
        fpm::auth::github::CALLBACK_URL => fpm::auth::github::token(req).await,
        fpm::auth::telegram::CALLBACK_URL => fpm::auth::telegram::token(req).await,
        "/auth/logout/" => logout(req),
        _ => Ok(actix_web::HttpResponse::new(
            actix_web::http::StatusCode::NOT_FOUND,
        )),
    }
}
