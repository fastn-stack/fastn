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

// route: /auth/login/
pub async fn login(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    // GitHub will be redirect to this url after login process completed
    let redirect_url: String = format!(
        "{}://{}/auth/github/",
        req.connection_info().scheme(),
        req.connection_info().host()
    );

    // Set up the config for the Github OAuth2 process.
    let client =
        utils::github_client().set_redirect_uri(oauth2::RedirectUrl::new(redirect_url.clone())?);

    // Note: public_repos user:email all these things are github resources
    // So we have to tell client who is getting logged in what are we going to access
    let (authorize_url, _token) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .url();

    // send redirect to /auth/github/
    Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, authorize_url.to_string()))
        .finish())
}

// handle: /auth/github/
pub async fn auth(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    let query = actix_web::web::Query::<AuthRequest>::from_query(req.query_string())?.0;
    let auth_url = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        "/auth/github/"
    );
    let client = utils::github_client().set_redirect_uri(oauth2::RedirectUrl::new(auth_url)?);
    match client
        .exchange_code(oauth2::AuthorizationCode::new(query.code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(token) => {
            let access_token = oauth2::TokenResponse::access_token(&token).secret();
            return Ok(actix_web::HttpResponse::Found()
                .cookie(
                    actix_web::cookie::Cookie::build("access_token", access_token.as_str())
                        .domain(fpm::auth::utils::domain(req.connection_info().host()))
                        .path("/")
                        .permanent()
                        .finish(),
                )
                .append_header((actix_web::http::header::LOCATION, "/".to_string()))
                .finish());
        }
        Err(err) => {
            dbg!(&err);
            Ok(actix_web::HttpResponse::InternalServerError().body(err.to_string()))
        }
    }
}

pub fn logout(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build("access_token", "")
                .domain(fpm::auth::utils::domain(req.connection_info().host()))
                .path("/")
                .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
                .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, "/".to_string()))
        .finish())
}

pub async fn get_starred_repo(
    access_token: &str,
    repo_list: &[fpm::user_group::UserIdentity],
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
pub async fn get_identity_fpm(
    req: actix_web::HttpRequest,
    _identities: &Vec<UserIdentity>,
) -> actix_web::HttpResponse {
    let _user_email_val: String = String::from("");
    let _user_login_val: String = String::from("");

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

        return actix_web::HttpResponse::BadRequest()
            .content_type("application/json")
            .json("BAD Request".to_string());
        // let mut all_found_repo: String = String::from("");
        // let reporesp = get_starred_repo(access_token_val.as_str(), identities.as_slice()).await;
        // match reporesp {
        //     Ok(reporesp) => {
        //         if reporesp.len() > 0 {
        //             for repo in reporesp {
        //                 if all_found_repo == "" {
        //                     all_found_repo = format!("{}{}", "github-starred:", repo);
        //                 } else {
        //                     all_found_repo = format!("{}{}{}", all_found_repo, ",", repo);
        //                 }
        //             }
        //         } else {
        //             all_found_repo = String::from("");
        //         }
        //
        //         let html = format!(
        //             r#"<html>
        //     <head><title>FDM</title></head>
        //     <body>
        //     github-username:{}<br/>gmail-email:{}<br/>{}
        //     </body>
        // </html>"#,
        //             user_login_val.clone(),
        //             user_email_val.clone(),
        //             all_found_repo.clone(),
        //         );
        //
        //         actix_web::HttpResponse::Ok().body(html)
        //     }
        //     Err(e) => {
        //
        //     }
        // }
    } else {
        return actix_web::HttpResponse::BadRequest()
            .content_type("application/json")
            .json("No record found.");
    }
}
