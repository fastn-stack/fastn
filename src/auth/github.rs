static GITHUB_CLIENT_ID_GLB: once_cell::sync::OnceCell<oauth2::ClientId> = once_cell::sync::OnceCell::new();
static GITHUB_CLIENT_SECRET_GLB: once_cell::sync::OnceCell<oauth2::ClientSecret> = once_cell::sync::OnceCell::new();
static AUTH_URL_GLB: once_cell::sync::OnceCell<oauth2::AuthUrl> = once_cell::sync::OnceCell::new();
static TOKEN_URL_GLB: once_cell::sync::OnceCell<oauth2::TokenUrl> = once_cell::sync::OnceCell::new();

use oauth2::TokenResponse;
#[derive(Debug)]
pub struct RepoObj {
    pub repo_owner:String,
    pub repo_title:String
}
pub fn index(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    
    let mut link="";
    let mut link_title="";
    match req.cookie("access_token"){
        Some(val)=>{
            //dbg!(val.value());
            link="auth/logout/";
            link_title="Logout";
        }
        None=>{
            link="auth/login/";
            link_title="Login";
        }
    }
    let welcome_msg;
    match req.cookie("user_login"){
        Some(val)=>{
            //dbg!(val.value());
            welcome_msg=format!("{}{}","Hello ",val.value());
        }
        None=>{
            welcome_msg=String::from("Welcome. Please first login: ");
        }
    }
    let html = format!(
        r#"<html>
        <head><title>FDM</title></head>
        <body>
            {} <a href="/{}">{}</a>
        </body>
    </html>"#,
    welcome_msg,
        link,
        link_title
    );

    actix_web::HttpResponse::Ok().body(html)
}
pub async fn login(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
     if GITHUB_CLIENT_ID_GLB.get().is_none(){
        GITHUB_CLIENT_ID_GLB.set(oauth2::ClientId::new(
            "77c964a9f6a7106a5a0e".to_string()
        )).unwrap();
     }  
     if GITHUB_CLIENT_SECRET_GLB.get().is_none(){
    GITHUB_CLIENT_SECRET_GLB.set(oauth2::ClientSecret::new(
        "916d6cc2e912082f89891120b929680494467ba6".to_string()
    )).unwrap();
}
if AUTH_URL_GLB.get().is_none(){
    AUTH_URL_GLB.set(oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string())
    .expect("Invalid authorization endpoint URL")).unwrap();
}
if TOKEN_URL_GLB.get().is_none(){
    TOKEN_URL_GLB.set(oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
    .expect("Invalid token endpoint URL")).unwrap();
}
        // Set up the config for the Github OAuth2 process.
        let client = oauth2::basic::BasicClient::new(
            GITHUB_CLIENT_ID_GLB.get().unwrap().to_owned(),
            Some(GITHUB_CLIENT_SECRET_GLB.get().unwrap().to_owned()),
            AUTH_URL_GLB.get().unwrap().to_owned(),
            Some(TOKEN_URL_GLB.get().unwrap().to_owned()),
        )
        .set_redirect_uri(
            oauth2::RedirectUrl::new("http://localhost:8000/auth/auth/".to_string()).expect("Invalid redirect URL"),
        );
    let authorize_url = client
    .authorize_url(oauth2::CsrfToken::new_random)
    .add_scope(oauth2::Scope::new("public_repo".to_string()))
    .add_scope(oauth2::Scope::new("user:email".to_string()))
    .url();
    actix_web::HttpResponse::Found().cookie(actix_web::cookie::Cookie::build("testcookie", "test").finish())
    .append_header((actix_web::http::header::LOCATION, authorize_url.0.to_string()))
    .finish()
   // HttpResponse::Ok().body(format!("username:"))
}

pub fn logout(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    //GITHUB_CLIENT_ID_GLB.
actix_web::HttpResponse::Found()
.cookie(
    actix_web::cookie::Cookie::build("user_login", "")
    .domain("localhost")
    .path("/auth/")
    .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
    .finish())
.cookie(
    actix_web::cookie::Cookie::build("user_email", "")
.domain("localhost")
.path("/auth/")
.expires(actix_web::cookie::time::OffsetDateTime::now_utc())
.finish())

.cookie(
    actix_web::cookie::Cookie::build("user_fullname", "")
    .domain("localhost")
    .path("/auth/")
    .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
    .finish())
.cookie(
    actix_web::cookie::Cookie::build("access_token", "")
    .domain("localhost")
    .path("/auth/")
    .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
    .finish())
    .cookie(
        actix_web::cookie::Cookie::build("user_login", "")
        .domain("localhost")
        .path("/get-identities/")
        .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
        .finish())
    .cookie(
        actix_web::cookie::Cookie::build("user_email", "")
    .domain("localhost")
    .path("/get-identities/")
    .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
.finish())
   
    .cookie(
        actix_web::cookie::Cookie::build("user_fullname", "")
        .domain("localhost")
        .path("/get-identities/")
        .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
        .finish())
    .cookie(
        actix_web::cookie::Cookie::build("access_token", "")
        .domain("localhost")
        .path("/get-identities/")
        .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
        .finish())
    .append_header((actix_web::http::header::LOCATION, "/auth/".to_string()))
    .finish()
}

pub async fn get_identity(req: actix_web::HttpRequest,repo_list:&Vec<String>) -> actix_web::HttpResponse {

    let user_email_val:String;
    let user_login_val:String;
    let access_token_val:String;
    let access_token = req.cookie("access_token");
    let user_login = req.cookie("user_login");
    let user_email = req.cookie("user_email");
    let user_fullname = req.cookie("user_fullname");
    match req.cookie("user_login"){
        Some(val)=>{
            //dbg!(val.value());
            user_login_val=val.value().to_string();
        }
        None=>{
            user_login_val=String::from("");
        }
    }
    match req.cookie("access_token"){
        Some(val)=>{
            //dbg!(val.value());
            access_token_val=val.value().to_string();
        }
        None=>{
            access_token_val=String::from("");
        }
    }
    match req.cookie("user_email"){
        Some(val)=>{
           // dbg!(val.value());
            user_email_val=val.value().to_string();
        }
        None=>{
            user_email_val=String::from("");
        }
    }
if !access_token_val.is_empty() {
let mut all_found_repo:String=String::from("");
    let reporesp=get_starred_repo(access_token_val.clone(),&repo_list).await;
    match reporesp {
        Ok(reporesp) => {
if reporesp.len()>0{
    for repo in reporesp{
        //all_found_repo
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

}

#[derive(serde::Deserialize)]
pub struct AuthRequest {
    pub code: String,
    pub state: String,
}
pub async fn auth(
    req: actix_web::HttpRequest,params: AuthRequest,
    ) -> actix_web::HttpResponse {
    
        let client = oauth2::basic::BasicClient::new(
            GITHUB_CLIENT_ID_GLB.get().unwrap().to_owned(),
            Some(GITHUB_CLIENT_SECRET_GLB.get().unwrap().to_owned()),
            AUTH_URL_GLB.get().unwrap().to_owned(),
            Some(TOKEN_URL_GLB.get().unwrap().to_owned()),
        )
        .set_redirect_uri(
            oauth2::RedirectUrl::new("http://localhost:8000/auth/auth/".to_string()).expect("Invalid redirect URL"),
        );    
    
    let code = oauth2::AuthorizationCode::new(params.code.clone());
    let _state = oauth2::CsrfToken::new(params.state.clone());
    let access_token;
    //let token_res = &data.oauth
    let token_res = client    
    .exchange_code(code)
        .request_async(oauth2::reqwest::async_http_client)
        .await;
        if let Ok(token) = token_res {
            access_token=token.access_token().clone().secret().to_string();
            //dbg!(access_token.clone());
            let userresp=user_details(access_token.clone()).await;
            match userresp {
                Ok(userresp) => {
                   
                    actix_web::HttpResponse::Found()
                    .cookie(
                        actix_web::cookie::Cookie::build("user_login", userresp.get("login").clone().unwrap().as_str().unwrap())
                        .domain("localhost")
                        .path("/auth/")
                        .permanent()
                        .finish())
                    .cookie(
                        actix_web::cookie::Cookie::build("user_email", userresp.get("email").clone().unwrap().as_str().unwrap())
                    .domain("localhost")
                    .path("/auth/")
                    .permanent().finish())
                   
                    .cookie(
                        actix_web::cookie::Cookie::build("user_fullname", userresp.get("name").clone().unwrap().as_str().unwrap())
                        .domain("localhost")
                        .path("/auth/")
                        .permanent()
                        .finish())
                    .cookie(
                        actix_web::cookie::Cookie::build("access_token", access_token.as_str())
                        .domain("localhost")
                        .path("/auth/")
                        .permanent()
                        .finish())
                        .cookie(
                            actix_web::cookie::Cookie::build("user_login", userresp.get("login").clone().unwrap().as_str().unwrap())
                            .domain("localhost")
                            .path("/get-identities/")
                            .permanent()
                            .finish())
                        .cookie(
                            actix_web::cookie::Cookie::build("user_email", userresp.get("email").clone().unwrap().as_str().unwrap())
                        .domain("localhost")
                        .path("/get-identities/")
                        .permanent().finish())
                       
                        .cookie(
                            actix_web::cookie::Cookie::build("user_fullname", userresp.get("name").clone().unwrap().as_str().unwrap())
                            .domain("localhost")
                            .path("/get-identities/")
                            .permanent()
                            .finish())
                        .cookie(
                            actix_web::cookie::Cookie::build("access_token", access_token.as_str())
                            .domain("localhost")
                            .path("/get-identities/")
                            .permanent()
                            .finish())
                  
                    .append_header((actix_web::http::header::LOCATION, "/auth/".to_string()))
                    .finish()
                }
                Err(_) => {
                   
                    actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, "/auth/".to_string()))
        .finish()
            }
            }
        }else{

            actix_web::HttpResponse::Found()
            .append_header((actix_web::http::header::LOCATION, "/auth/".to_string()))
            .finish()
        }
       
    }
async fn user_details(access_token:String) -> Result<serde_json::value::Value,reqwest::Error> {

    let token_val=format!("{}{}", String::from("token "), access_token);

    let request_obj=reqwest::Client::new()
        .get("https://api.github.com/user")
        .header(reqwest::header::AUTHORIZATION, token_val.clone())
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("fpm"))
        .send()
        .await?;
        let resp: serde_json::Value = request_obj.json().await?;
        Ok(resp)
    }
    async fn get_starred_repo(access_token:String,repo_list:&Vec<String>) -> Result<Vec<String>,reqwest::Error> {
        let token_val=format!("{}{}", String::from("Bearer "), access_token);
        let mut starred_repo:Vec<String>=vec![];

        let api_url=format!("{}", String::from("https://api.github.com/user/starred"));
        let request_obj=reqwest::Client::new()
        .get(api_url.clone())
        .header(reqwest::header::AUTHORIZATION, token_val)
        .header(reqwest::header::ACCEPT, "application/json")
        .header(reqwest::header::USER_AGENT, reqwest::header::HeaderValue::from_static("fpm"))
        .send()
        .await?;
        let resp:serde_json::Value = request_obj.json().await?;
       
        if resp.as_array().unwrap().len()>0
        {
        for repo in repo_list{
        for respobj in resp.as_array().unwrap().iter(){
        if repo==respobj.get("full_name").unwrap(){
           starred_repo.push(respobj.get("full_name").unwrap().to_string());
        }
        }
        }
        }
        Ok(starred_repo)
     
        
    }