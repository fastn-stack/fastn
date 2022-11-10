async fn get_identities_route(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    let identities = get_identities(req.clone());
    let identity_obj = fpm::auth::github::get_identity_fpm(req, &identities);
    Ok(identity_obj.await)
}

fn get_identities(req: actix_web::HttpRequest) -> Vec<fpm::auth::github::UserIdentity> {
    let base_url = format!(
        "{}://{}",
        req.connection_info().scheme(),
        req.connection_info().host()
    );

    let mut repo_list: Vec<fpm::auth::github::UserIdentity> = Vec::new();
    let uri_string = req.uri();
    let final_url: String = format!("{}{}", base_url.clone(), uri_string.clone().to_string());
    let request_url = url::Url::parse(&final_url.to_string()).unwrap();
    let pairs = request_url.query_pairs();
    for pair in pairs {
        repo_list.push(fpm::auth::github::UserIdentity {
            key: pair.0.to_string(),
            value: pair.1.to_string(),
        });
    }
    repo_list
}

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
    } else if req.path() == "/auth/github/" {
        // route will be called from after github login redirected request by passing code and state
        return fpm::auth::github::auth(req).await;
    } else if req.path() == "/auth/logout/" {
        return fpm::auth::github::logout(req);
    } else if req.path() == "/auth/get-identities/" {
        return get_identities_route(req).await;
    }
    return Ok(actix_web::HttpResponse::new(
        actix_web::http::StatusCode::NOT_FOUND,
    ));
}
