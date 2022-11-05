static GITHUB_CLIENT_ID_GLB: once_cell::sync::OnceCell<oauth2::ClientId> = once_cell::sync::OnceCell::new();
static GITHUB_CLIENT_SECRET_GLB: once_cell::sync::OnceCell<oauth2::ClientSecret> = once_cell::sync::OnceCell::new();
static AUTH_URL_GLB: once_cell::sync::OnceCell<oauth2::AuthUrl> = once_cell::sync::OnceCell::new();
static TOKEN_URL_GLB: once_cell::sync::OnceCell<oauth2::TokenUrl> = once_cell::sync::OnceCell::new();

#[derive(Debug)]
pub struct RepoObj {
    pub repo_owner:String,
    pub repo_title:String
}
pub fn index(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    dotenv::dotenv();
    //actix_web::
    dbg!(req.connection_info());
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
    let base_url=format!("{}{}{}",req.connection_info().scheme(),"://",req.connection_info().host());    
    let auth_url=format!("{}{}",base_url,"/auth/auth/");
     if GITHUB_CLIENT_ID_GLB.get().is_none(){
        GITHUB_CLIENT_ID_GLB.set(oauth2::ClientId::new(
            std::env::var("GITHUB_CLIENT_ID").unwrap()
        )).unwrap();
     }  
     if GITHUB_CLIENT_SECRET_GLB.get().is_none(){
        GITHUB_CLIENT_SECRET_GLB.set(oauth2::ClientSecret::new(
            std::env::var("GITHUB_CLIENT_SECRET").unwrap()
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
            oauth2::RedirectUrl::new(auth_url.clone()).expect("Invalid redirect URL"),
        );
    let authorize_url = client
    .authorize_url(oauth2::CsrfToken::new_random)
    .add_scope(oauth2::Scope::new("public_repo".to_string()))
    .add_scope(oauth2::Scope::new("user:email".to_string()))
    .url();
    actix_web::HttpResponse::Found().cookie(actix_web::cookie::Cookie::build("testcookie", "test").finish())
    .append_header((actix_web::http::header::LOCATION, authorize_url.0.to_string()))
    .finish()

}

pub fn logout(req: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let connection_obj=req.connection_info().clone();
    let domain;
    if connection_obj.host().contains("localhost"){
        domain="localhost";
    }else{
        domain=connection_obj.host();
    }
actix_web::HttpResponse::Found()
.cookie(
    actix_web::cookie::Cookie::build("user_login", "")
    .domain(domain)
    .path("/auth/")
    .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
    .finish())
.cookie(
    actix_web::cookie::Cookie::build("user_email", "")
.domain(domain)
.path("/auth/")
.expires(actix_web::cookie::time::OffsetDateTime::now_utc())
.finish())

.cookie(
    actix_web::cookie::Cookie::build("user_fullname", "")
    .domain(domain)
    .path("/auth/")
    .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
    .finish())
.cookie(
    actix_web::cookie::Cookie::build("access_token", "")
    .domain(domain)
    .path("/auth/")
    .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
    .finish())
    .cookie(
        actix_web::cookie::Cookie::build("user_login", "")
        .domain(domain)
        .path("/get-identities/")
        .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
        .finish())
    .cookie(
        actix_web::cookie::Cookie::build("user_email", "")
    .domain(domain)
    .path("/get-identities/")
    .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
.finish())

    .cookie(
        actix_web::cookie::Cookie::build("user_fullname", "")
        .domain(domain)
        .path("/get-identities/")
        .expires(actix_web::cookie::time::OffsetDateTime::now_utc())
        .finish())
    .cookie(
        actix_web::cookie::Cookie::build("access_token", "")
        .domain(domain)
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
           
            user_login_val=val.value().to_string();
        }
        None=>{
            user_login_val=String::from("");
        }
    }
    match req.cookie("access_token"){
        Some(val)=>{
           
            access_token_val=val.value().to_string();
        }
        None=>{
            access_token_val=String::from("");
        }
    }
    match req.cookie("user_email"){
        Some(val)=>{
           
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
        let connection_obj=req.connection_info().clone();
    let domain;
    if connection_obj.host().contains("localhost"){
        domain="localhost";
    }else{
        domain=connection_obj.host();
    }
        let base_url=format!("{}{}{}",req.connection_info().scheme(),"://",req.connection_info().host());    
        let auth_url=format!("{}{}",base_url,"/auth/auth/");
        let client = oauth2::basic::BasicClient::new(
            GITHUB_CLIENT_ID_GLB.get().unwrap().to_owned(),
            Some(GITHUB_CLIENT_SECRET_GLB.get().unwrap().to_owned()),
            AUTH_URL_GLB.get().unwrap().to_owned(),
            Some(TOKEN_URL_GLB.get().unwrap().to_owned()),
        )
        .set_redirect_uri(
            oauth2::RedirectUrl::new(auth_url.clone()).expect("Invalid redirect URL"),
        );    
    
    let code = oauth2::AuthorizationCode::new(params.code.clone());
    let _state = oauth2::CsrfToken::new(params.state.clone());
    let access_token;
    let access_token_obj;
    let token_res = client    
    .exchange_code(code)
        .request_async(oauth2::reqwest::async_http_client)
        .await;
        if let Ok(token) = token_res {
            access_token_obj=oauth2::TokenResponse::access_token(&token);
            //dbg!(access_token_obj.clone().secret().to_string());
            access_token=access_token_obj.clone().secret().to_string();
            //access_token=token.access_token().clone().secret().to_string();
            
            let userresp=user_details(access_token.clone()).await;
            match userresp {
                Ok(userresp) => {
                   
                    actix_web::HttpResponse::Found()
                    .cookie(
                        actix_web::cookie::Cookie::build("user_login", userresp.get("login").clone().unwrap().as_str().unwrap())
                        .domain(domain)
                        .path("/auth/")
                        .permanent()
                        .finish())
                    .cookie(
                        actix_web::cookie::Cookie::build("user_email", userresp.get("email").clone().unwrap().as_str().unwrap())
                    .domain(domain)
                    .path("/auth/")
                    .permanent().finish())
                   
                    .cookie(
                        actix_web::cookie::Cookie::build("user_fullname", userresp.get("name").clone().unwrap().as_str().unwrap())
                        .domain(domain)
                        .path("/auth/")
                        .permanent()
                        .finish())
                    .cookie(
                        actix_web::cookie::Cookie::build("access_token", access_token.as_str())
                        .domain(domain)
                        .path("/auth/")
                        .permanent()
                        .finish())
                        .cookie(
                            actix_web::cookie::Cookie::build("user_login", userresp.get("login").clone().unwrap().as_str().unwrap())
                            .domain(domain)
                            .path("/get-identities/")
                            .permanent()
                            .finish())
                        .cookie(
                            actix_web::cookie::Cookie::build("user_email", userresp.get("email").clone().unwrap().as_str().unwrap())
                        .domain(domain)
                        .path("/get-identities/")
                        .permanent().finish())
                       
                        .cookie(
                            actix_web::cookie::Cookie::build("user_fullname", userresp.get("name").clone().unwrap().as_str().unwrap())
                            .domain(domain)
                            .path("/get-identities/")
                            .permanent()
                            .finish())
                        .cookie(
                            actix_web::cookie::Cookie::build("access_token", access_token.as_str())
                            .domain(domain)
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