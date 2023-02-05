// route: /auth/login/
pub async fn login(
    req: actix_web::HttpRequest,
    edition: Option<String>,
    external_js: Vec<String>,
    inline_js: Vec<String>,
    external_css: Vec<String>,
    inline_css: Vec<String>,
) -> fastn_core::Result<actix_web::HttpResponse> {
    if fastn_core::auth::utils::is_login(&req) {
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
        "github" => fastn_core::auth::github::login(req).await,
        "telegram" => fastn_core::auth::telegram::login(req).await,
        "discord" => fastn_core::auth::discord::login(req).await,
        "twitter" => fastn_core::auth::twitter::login(req).await,
        // TODO: Remove this after demo
        _ => {
            let mut req = fastn_core::http::Request::from_actix(req, actix_web::web::Bytes::new());
            req.path = "/sorry/".to_string();
            fastn_core::commands::serve::serve(
                req,
                edition,
                external_js,
                inline_js,
                external_css,
                inline_css,
            )
            .await
        } // _ => unreachable!(),
    }
}

// route: /auth/logout/
pub fn logout(req: actix_web::HttpRequest) -> fastn_core::Result<actix_web::HttpResponse> {
    // TODO: Refactor, Not happy with this code, too much of repetition of similar code
    // It is logging out from all the platforms

    // Ideally it should capture the platform in the request and then logged out
    // only from that platform
    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::GitHub.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::TeleGram.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Slack.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Discord.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Google.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Amazon.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Apple.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Baidu.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::BitBucket.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::DigitalOcean.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::DoorKeeper.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::DropBox.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Facebook.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::GitLab.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Instagram.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::LinkedIn.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Microsoft.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Okta.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Pintrest.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::TikTok.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Twitch.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Twitter.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::WeChat.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Yahoo.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .cookie(
            actix_web::cookie::Cookie::build(fastn_core::auth::AuthProviders::Zoho.as_str(), "")
                .domain(fastn_core::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish())
}

// handle: if request.url starts with /auth/
#[tracing::instrument(skip_all)]
pub async fn handle_auth(
    req: actix_web::HttpRequest,
    edition: Option<String>,
    external_js: Vec<String>,
    inline_js: Vec<String>,
    external_css: Vec<String>,
    inline_css: Vec<String>,
) -> fastn_core::Result<fastn_core::http::Response> {
    match req.path() {
        "/auth/login/" => {
            login(
                req,
                edition,
                external_js,
                inline_js,
                external_css,
                inline_css,
            )
            .await
        }
        fastn_core::auth::github::CALLBACK_URL => fastn_core::auth::github::callback(req).await,
        fastn_core::auth::telegram::CALLBACK_URL => fastn_core::auth::telegram::token(req).await,
        fastn_core::auth::discord::CALLBACK_URL => fastn_core::auth::discord::callback(req).await,
        fastn_core::auth::twitter::CALLBACK_URL => fastn_core::auth::twitter::callback(req).await,
        "/auth/logout/" => logout(req),
        _ => Ok(actix_web::HttpResponse::new(
            actix_web::http::StatusCode::NOT_FOUND,
        )),
    }
}
