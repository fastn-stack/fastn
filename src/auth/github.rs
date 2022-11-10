// Lazy means a value which initialize at the first time access
// we have to access it before using it and make sure to use it while starting a server
// TODO: Make sure that before accessing in the API the are set
static GITHUB_CLIENT_ID: once_cell::sync::Lazy<oauth2::ClientId> = {
    once_cell::sync::Lazy::new(|| {
        oauth2::ClientId::new(
            std::env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID not set in env"),
        )
    })
};

static GITHUB_CLIENT_SECRET: once_cell::sync::Lazy<oauth2::ClientSecret> = {
    once_cell::sync::Lazy::new(|| {
        oauth2::ClientSecret::new(
            std::env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET not set in env"),
        )
    })
};

const TOKEN_URL: once_cell::sync::Lazy<oauth2::TokenUrl> = once_cell::sync::Lazy::new(|| {
    oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
        .expect("Invalid token endpoint URL")
});

const AUTH_URL: once_cell::sync::Lazy<oauth2::AuthUrl> = once_cell::sync::Lazy::new(|| {
    oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
        .expect("GitHub AuthUrl is wrong")
});

#[derive(Debug)]
pub struct RepoObj {
    pub repo_owner: String,
    pub repo_title: String,
}
#[derive(Debug, Clone, serde::Serialize)]
pub struct UserIdentity {
    pub key: String,
    pub value: String,
}

#[derive(serde::Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}

pub async fn login(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    // We have to redirect here to set access_token
    let redirect_url: String = format!(
        "{}://{}/auth/auth/",
        req.connection_info().scheme(),
        req.connection_info().host()
    );
    // Set up the config for the Github OAuth2 process.
    let client =
        utils::github_client().set_redirect_uri(oauth2::RedirectUrl::new(redirect_url.clone())?);
    // Note: public_repos user:email all these things are github resources
    // So we have to tell client who is getting logged in what are we going to access
    let authorize_url = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .url();

    Ok(actix_web::HttpResponse::Found()
        .append_header((
            actix_web::http::header::LOCATION,
            authorize_url.0.to_string(),
        ))
        .finish())
}

pub fn logout(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let connection_obj = req.connection_info().clone();
    let domain;
    let host_info = connection_obj.host();
    let domain_parts: Vec<&str> = host_info.split(":").collect();
    domain = domain_parts.get(0).unwrap();
    //let domain="localhost";
    actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build("access_token", "")
                .domain(domain.clone())
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, "/auth/".to_string()))
        .finish()
}

pub async fn auth(req: actix_web::HttpRequest, params: AuthRequest) -> actix_web::HttpResponse {
    //let domain = "localhost";
    let connection_obj = req.connection_info().clone();
    let domain;
    let host_info = connection_obj.host();
    let domain_parts: Vec<&str> = host_info.split(":").collect();
    domain = domain_parts.get(0).unwrap();
    //let base_url = "http://localhost:8000";
    let base_url = format!(
        "{}{}{}",
        req.connection_info().scheme(),
        "://",
        req.connection_info().host()
    );
    let auth_url = format!("{}{}", base_url, "/auth/auth/");
    let client = oauth2::basic::BasicClient::new(
        GITHUB_CLIENT_ID.to_owned(),
        Some(GITHUB_CLIENT_SECRET.to_owned()),
        AUTH_URL.clone(),
        Some(TOKEN_URL.clone()),
    )
    .set_redirect_uri(oauth2::RedirectUrl::new(auth_url.clone()).expect("Invalid redirect URL"));

    let code = oauth2::AuthorizationCode::new(params.code.clone());
    let _state = oauth2::CsrfToken::new(params.state.clone());
    let access_token;
    let access_token_obj;
    let token_res = client
        .exchange_code(code)
        .request_async(oauth2::reqwest::async_http_client)
        .await;
    if let Ok(token) = token_res {
        access_token_obj = oauth2::TokenResponse::access_token(&token);
        access_token = access_token_obj.clone().secret().to_string();
        let userresp = user_details(access_token.clone()).await;
        match userresp {
            Ok(userresp) => actix_web::HttpResponse::Found()
                .cookie(
                    actix_web::cookie::Cookie::build("access_token", access_token.as_str())
                        .domain(domain.clone())
                        .path("/")
                        .permanent()
                        .finish(),
                )
                .append_header((actix_web::http::header::LOCATION, "/".to_string()))
                .finish(),
            Err(_) => actix_web::HttpResponse::Found()
                .append_header((actix_web::http::header::LOCATION, "/auth/".to_string()))
                .finish(),
        }
    } else {
        actix_web::HttpResponse::Found()
            .append_header((actix_web::http::header::LOCATION, "/auth/".to_string()))
            .finish()
    }
}

async fn user_details(access_token: String) -> Result<serde_json::value::Value, reqwest::Error> {
    let token_val = format!("{}{}", String::from("token "), access_token);

    let request_obj = reqwest::Client::new()
        .get("https://api.github.com/user")
        .header(reqwest::header::AUTHORIZATION, token_val.clone())
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("fpm"),
        )
        .send()
        .await?;
    let resp: serde_json::Value = request_obj.json().await?;
    Ok(resp)
}

async fn get_starred_repo(
    access_token: &str,
    repo_list: &Vec<UserIdentity>,
) -> Result<Vec<String>, reqwest::Error> {
    let token_val = format!("{}{}", String::from("Bearer "), access_token);
    let mut starred_repo: Vec<String> = vec![];
    let api_url = format!("{}", String::from("https://api.github.com/user/starred"));
    let request_obj = reqwest::Client::new()
        .get(api_url.clone())
        .header(reqwest::header::AUTHORIZATION, token_val)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(
            reqwest::header::USER_AGENT,
            reqwest::header::HeaderValue::from_static("fpm"),
        )
        .send()
        .await?;

    // This should parse to Struct
    let resp: serde_json::Value = request_obj.json().await?;

    dbg!(&resp);

    // TODO: remove unwrap, refactor this code
    dbg!(&repo_list);
    if resp.as_array().unwrap().len() > 0 {
        for repo in repo_list {
            for respobj in resp.as_array().unwrap().iter() {
                dbg!(&repo.value);
                dbg!(&respobj.get("full_name").unwrap());

                if repo.key.eq("github-starred") && repo.value.eq(respobj.get("full_name").unwrap())
                {
                    let value =
                        serde_json::from_value(respobj.get("full_name").unwrap().clone()).unwrap();
                    starred_repo.push(value);
                }
            }
        }
    }
    Ok(starred_repo)
}

pub async fn get_identity_fpm(
    req: actix_web::HttpRequest,
    identities: &Vec<UserIdentity>,
) -> actix_web::HttpResponse {
    let user_email_val: String = String::from("");
    let user_login_val: String = String::from("");

    let access_token_val: String;

    match req.cookie("access_token") {
        Some(val) => {
            access_token_val = val.value().to_owned();
        }
        None => {
            access_token_val = String::from("");
        }
    }
    if !access_token_val.is_empty() {
        // let userresp = user_details(access_token_val.clone()).await;
        // match userresp {
        //     Ok(userresp) => {
        //         if userresp.get("login").is_some() {
        //             user_login_val = userresp.get("login").unwrap().to_string();
        //         }
        //         if userresp.get("email").is_some() {
        //             user_email_val = userresp.get("email").unwrap().to_string();
        //         }
        //     }
        //     Err(_) => {}
        // }

        let mut all_found_repo: String = String::from("");
        let reporesp = get_starred_repo(access_token_val.as_str(), identities).await;

        match reporesp {
            Ok(reporesp) => {
                if reporesp.len() > 0 {
                    for repo in reporesp {
                        if all_found_repo == "" {
                            all_found_repo = format!("{}{}", "github-starred:", repo);
                        } else {
                            all_found_repo = format!("{}{}{}", all_found_repo, ",", repo);
                        }
                    }
                } else {
                    all_found_repo = String::from("");
                }

                let html = format!(
                    r#"<html>
            <head><title>FDM</title></head>
            <body>
            github-username:{}<br/>gmail-email:{}<br/>{}
            </body>
        </html>"#,
                    user_login_val.clone(),
                    user_email_val.clone(),
                    all_found_repo.clone(),
                );

                actix_web::HttpResponse::Ok().body(html)
            }
            Err(e) => {
                return actix_web::HttpResponse::BadRequest()
                    .content_type("application/json")
                    .json(e.to_string());
            }
        }
    } else {
        return actix_web::HttpResponse::BadRequest()
            .content_type("application/json")
            .json("No record found.");
    }
}

// TODO: rename the method later
pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[(String, String)],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    dbg!(&cookies);

    let access_token = cookies.get("access_token").ok_or_else(|| {
        fpm::Error::GenericError("access_token not found in the cookies".to_string())
    })?;

    let identities = identities
        .into_iter()
        .map(|(k, v)| UserIdentity {
            key: k.to_string(),
            value: v.to_string(),
        })
        .collect();

    let user_starred_repos = get_starred_repo(access_token.as_str(), &identities).await?;

    dbg!(&user_starred_repos);

    Ok(user_starred_repos
        .into_iter()
        .map(|repo| fpm::user_group::UserIdentity {
            key: "github-starred".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn index(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let mut link = "auth/login/";
    let mut link_title = "Login";
    match req.cookie("access_token") {
        Some(val) => {
            if val.value().to_string() != "" {
                link = "auth/logout/";
                link_title = "Logout";
            }
        }
        None => {
            //link="auth/login/";
            // link_title="Login";
        }
    }
    let mut welcome_msg = String::from("Welcome. Please first login: ");

    match req.cookie("access_token") {
        Some(val) => {
            if val.value().to_string() != "" {
                let userresp = user_details(val.value().to_string()).await;
                match userresp {
                    Ok(userresp) => {
                        let user_login = userresp.get("login");
                        if userresp.get("login").is_some() {
                            welcome_msg =
                                format!("{}{}", "Hello ", user_login.clone().unwrap().to_string());
                        }
                    }
                    Err(_) => {}
                }
            }
        }
        None => {
            welcome_msg = String::from("Welcome. Please first login: ");
        }
    }
    let html = format!(
        r#"<html>
        <head><title>FPM</title></head>
        <body>
            {} <a href="/{}">{}</a>
        </body>
    </html>"#,
        welcome_msg, link, link_title
    );

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(html)
}

pub mod utils {
    pub fn github_client() -> oauth2::basic::BasicClient {
        use fpm::auth::github::{AUTH_URL, GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET, TOKEN_URL};
        oauth2::basic::BasicClient::new(
            GITHUB_CLIENT_ID.to_owned(),
            Some(GITHUB_CLIENT_SECRET.to_owned()),
            AUTH_URL.clone(),
            Some(TOKEN_URL.clone()),
        )
    }
}
