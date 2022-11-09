static GITHUB_CLIENT_ID_GLB: once_cell::sync::OnceCell<oauth2::ClientId> =
    once_cell::sync::OnceCell::new();
static GITHUB_CLIENT_SECRET_GLB: once_cell::sync::OnceCell<oauth2::ClientSecret> =
    once_cell::sync::OnceCell::new();
static AUTH_URL_GLB: once_cell::sync::OnceCell<oauth2::AuthUrl> = once_cell::sync::OnceCell::new();
static TOKEN_URL_GLB: once_cell::sync::OnceCell<oauth2::TokenUrl> =
    once_cell::sync::OnceCell::new();

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
pub async fn index(req: fpm::http::Request) -> actix_web::HttpResponse {
    dotenv::dotenv().ok();
    let mut link = "auth/login/";
    let mut link_title = "Login";
    match req.cookie("access_token") {
        Some(val) => {
            if val != "" {
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
            if val != "" {
                let userresp = user_details(val).await;
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
        <head><title>FDM</title></head>
        <body>
            {} <a href="/{}">{}</a>
        </body>
    </html>"#,
        welcome_msg, link, link_title
    );

    actix_web::HttpResponse::Ok().body(html)
}
pub async fn login(req: fpm::http::Request) -> actix_web::HttpResponse {
    // TODO: Need to remove it it should be part of while server is getting started
    dotenv::dotenv().ok();
    let auth_url: String = format!("{}://{}/auth/auth/", req.scheme(), req.host());
    if GITHUB_CLIENT_ID_GLB.get().is_none() {
        GITHUB_CLIENT_ID_GLB
            .set(oauth2::ClientId::new(
                std::env::var("GITHUB_CLIENT_ID").unwrap(),
            ))
            .unwrap();
    }
    if GITHUB_CLIENT_SECRET_GLB.get().is_none() {
        GITHUB_CLIENT_SECRET_GLB
            .set(oauth2::ClientSecret::new(
                std::env::var("GITHUB_CLIENT_SECRET").unwrap(),
            ))
            .unwrap();
    }
    if AUTH_URL_GLB.get().is_none() {
        AUTH_URL_GLB
            .set(
                oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
                    .expect("Invalid authorization endpoint URL"),
            )
            .unwrap();
    }
    if TOKEN_URL_GLB.get().is_none() {
        TOKEN_URL_GLB
            .set(
                oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                    .expect("Invalid token endpoint URL"),
            )
            .unwrap();
    }
    // Set up the config for the Github OAuth2 process.
    let client = oauth2::basic::BasicClient::new(
        GITHUB_CLIENT_ID_GLB.get().unwrap().to_owned(),
        Some(GITHUB_CLIENT_SECRET_GLB.get().unwrap().to_owned()),
        AUTH_URL_GLB.get().unwrap().to_owned(),
        Some(TOKEN_URL_GLB.get().unwrap().to_owned()),
    )
    .set_redirect_uri(oauth2::RedirectUrl::new(auth_url.clone()).expect("Invalid redirect URL"));
    let authorize_url = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .url();
    actix_web::HttpResponse::Found()
        .append_header((
            actix_web::http::header::LOCATION,
            authorize_url.0.to_string(),
        ))
        .finish()
}

pub async fn logout(req: fpm::http::Request) -> actix_web::HttpResponse {
    /*let connection_obj=req.connection_info().clone();
    let domain;
    let host_info=connection_obj.host();
    let domain_parts:Vec<&str>=host_info.split(":").collect();
    domain=domain_parts.get(0).unwrap();*/
    let domain = "localhost";
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

pub async fn get_identity(req: fpm::http::Request) -> actix_web::HttpResponse {
    let mut user_email_val: String = String::from("");
    let mut user_login_val: String = String::from("");
    let access_token_val: String;
    let access_token = req.cookie("access_token");

    //let base_url=format!("{}{}{}",req.connection_info().scheme(),"://",req.connection_info().host());
    let base_url = "http://localhost:8000";
    let mut repo_list: Vec<String> = Vec::new();
    let uri_string = req.uri();
    let final_url: String = format!("{}{}", base_url.clone(), uri_string.clone().to_string());
    let request_url = url::Url::parse(&final_url.to_string()).unwrap();
    let pairs = request_url.query_pairs();
    for pair in pairs {
        if pair.0 == "github_starred" {
            if !repo_list.contains(&pair.1.to_string()) {
                repo_list.push(pair.1.to_string());
            }
        }
    }
    match req.cookie("access_token") {
        Some(val) => {
            //access_token_val=val.value().to_string();
            access_token_val = val;
        }
        None => {
            access_token_val = String::from("");
        }
    }
    if !access_token_val.is_empty() {
        let userresp = user_details(access_token_val.clone()).await;
        match userresp {
            Ok(userresp) => {
                //let user_login=userresp.get("login");
                if userresp.get("login").is_some() {
                    user_login_val = userresp.get("login").unwrap().to_string();
                }
                if userresp.get("email").is_some() {
                    user_email_val = userresp.get("email").unwrap().to_string();
                }
            }
            Err(_) => {}
        }
        let mut all_found_repo: String = String::from("");
        let reporesp = get_starred_repo(access_token_val.clone(), &repo_list).await;
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

#[derive(serde::Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}
pub async fn auth(req: fpm::http::Request, params: AuthRequest) -> actix_web::HttpResponse {
    dotenv::dotenv().ok();
    let domain = "localhost";
    let base_url = "http://localhost:8000";
    let auth_url = format!("{}{}", base_url, "/auth/auth/");
    let client = oauth2::basic::BasicClient::new(
        GITHUB_CLIENT_ID_GLB.get().unwrap().to_owned(),
        Some(GITHUB_CLIENT_SECRET_GLB.get().unwrap().to_owned()),
        AUTH_URL_GLB.get().unwrap().to_owned(),
        Some(TOKEN_URL_GLB.get().unwrap().to_owned()),
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
                .append_header((actix_web::http::header::LOCATION, "/auth/".to_string()))
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
    access_token: String,
    repo_list: &Vec<String>,
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
    let resp: serde_json::Value = request_obj.json().await?;

    if resp.as_array().unwrap().len() > 0 {
        for repo in repo_list {
            for respobj in resp.as_array().unwrap().iter() {
                if repo == respobj.get("full_name").unwrap() {
                    starred_repo.push(respobj.get("full_name").unwrap().to_string());
                }
            }
        }
    }
    Ok(starred_repo)
}

/*pub async fn get_identity_fpm(cookies:&std::collections::HashMap<String, String>,identities: &Vec<UserIdentity>) -> actix_web::HttpResponse {
        let mut user_email_val:String=String::from("");
        let mut user_login_val:String=String::from("");
        dbg!(cookies);
   dbg!(identities);

    let access_token_val:String;
    let access_token = cookies.get("access_token");

    match cookies.get("access_token"){
        Some(val)=>{

            //access_token_val=val.value().to_string();
            access_token_val=val.to_owned();
        }
        None=>{
            access_token_val=String::from("");
        }
    }
if !access_token_val.is_empty() {
    let userresp=user_details(access_token_val.clone()).await;
    match userresp {
        Ok(userresp) => {
            if userresp.get("login").is_some(){
                user_login_val=userresp.get("login").unwrap().to_string();
            }
            if userresp.get("email").is_some(){
                user_email_val=userresp.get("email").unwrap().to_string();
            }

        }Err(_) => {

    }
}
let mut all_found_repo:String=String::from("");
    let reporesp=get_starred_repo(access_token_val.clone(),&repo_list).await;
    //let reporesp;
    match reporesp {
        Ok(reporesp) => {
if reporesp.len()>0{
    for repo in reporesp{
        if all_found_repo==""{
            all_found_repo=format!("{}{}","github-starred:",repo);
        }else{
            all_found_repo=format!("{}{}{}",all_found_repo,",",repo);
        }

    }

}else{
    all_found_repo=String::from("");
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
            return actix_web::HttpResponse::BadRequest().content_type("application/json")
            .json(e.to_string());
    }
    }


}else{
    return actix_web::HttpResponse::BadRequest().content_type("application/json")
        .json("No record found.");
}

}*/
