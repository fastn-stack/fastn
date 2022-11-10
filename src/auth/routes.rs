async fn auth_route(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    Ok(fpm::auth::github::index(req).await)
}

async fn login_route(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    Ok(fpm::auth::github::login(req).await)
}

async fn logout_route(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    Ok(fpm::auth::github::logout(req))
}

async fn auth_auth_route(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    let base_url = format!(
        "{}{}{}",
        req.connection_info().scheme(),
        "://",
        req.connection_info().host()
    );
    //let base_url="http://localhost:8000";
    let uri_string = req.uri();
    let final_url: String = format!("{}{}", base_url.clone(), uri_string.clone().to_string());
    let request_url = url::Url::parse(&final_url.to_string()).unwrap();
    let pairs = request_url.query_pairs();
    let mut code = String::from("");
    let mut state = String::from("");
    for pair in pairs {
        if pair.0 == "code" {
            code = pair.1.to_string();
        }
        if pair.0 == "state" {
            state = pair.1.to_string();
        }
    }
    let auth_obj = fpm::auth::github::auth(
        req,
        fpm::auth::github::AuthRequest {
            code: code.clone(),
            state: state.clone(),
        },
    );
    Ok(auth_obj.await)
}

async fn get_identities_route(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    let identities = get_identities(req.clone());
    let identity_obj = fpm::auth::github::get_identity_fpm(req, &identities);
    Ok(identity_obj.await)
}
fn get_identities(req: actix_web::HttpRequest) -> Vec<fpm::auth::github::UserIdentity> {
    //let mut repo_list: Vec<fpm::auth::github::UserIdentity> = Vec::new();
    let base_url = format!(
        "{}{}{}",
        req.connection_info().scheme(),
        "://",
        req.connection_info().host()
    );
    //let base_url=format!("{}{}{}",req.host(),"://",req.host());
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
        return auth_route(req.clone()).await;
    } else if req.path() == "/auth/login/" {
        return login_route(req.clone()).await;
    } else if req.path() == "/auth/logout/" {
        return logout_route(req.clone()).await;
    } else if req.path() == "/auth/auth/" {
        return auth_auth_route(req.clone()).await;
    } else if req.path() == "/auth/get-identities/" {
        return get_identities_route(req.clone()).await;
    }
    return Ok(actix_web::HttpResponse::new(
        actix_web::http::StatusCode::NOT_FOUND,
    ));
}
