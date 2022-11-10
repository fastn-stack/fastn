pub async fn handle_auth(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    if req.path() == "/auth/" {
        // TODO: this is not required we can remove it.
        return Ok(fpm::auth::github::index(req).await);
    } else if req.path() == "/auth/login/" {
        // TODO: It need paas it as query parameters
        let platform = "github";
        return match platform {
            "github" => fpm::auth::github::login(req).await,
            "discord" => unreachable!(),
            _ => unreachable!(),
        };
    } else if req.path() == "/auth/github/access/" {
        // route will be called from after github login redirected request by passing code and state
        return fpm::auth::github::access_token(req).await;
    } else if req.path() == "/auth/logout/" {
        return fpm::auth::github::logout(req);
    }
    return Ok(actix_web::HttpResponse::new(
        actix_web::http::StatusCode::NOT_FOUND,
    ));
}
