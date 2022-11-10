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

// TODO: This has be set while creating the GitHub OAuth Application
pub const ACCESS_URL: &str = "/auth/github/access/";

// route: /auth/login/
pub async fn login(req: actix_web::HttpRequest) -> fpm::Result<fpm::http::Response> {
    // GitHub will be redirect to this url after login process completed
    let redirect_url: String = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        ACCESS_URL
    );

    // Set up the config for the Github OAuth2 process.
    let client = utils::github_client().set_redirect_uri(oauth2::RedirectUrl::new(redirect_url)?);
    // Note: public_repos user:email all these things are github resources
    // So we have to tell client who is getting logged in what are we going to access
    let (authorize_url, _token) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .url();
    // send redirect to /auth/github/access/
    Ok(actix_web::HttpResponse::Found()
        .append_header((actix_web::http::header::LOCATION, authorize_url.to_string()))
        .finish())
}

// route: /auth/github/access/
// In this API we are accessing
// the token and setting it to cookies
pub async fn access_token(req: actix_web::HttpRequest) -> fpm::Result<actix_web::HttpResponse> {
    #[derive(serde::Deserialize)]
    pub struct QueryParams {
        pub code: String,
        pub state: String,
    }

    let query = actix_web::web::Query::<QueryParams>::from_query(req.query_string())?.0;
    let auth_url = format!(
        "{}://{}{}",
        req.connection_info().scheme(),
        req.connection_info().host(),
        ACCESS_URL
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
                    actix_web::cookie::Cookie::build(
                        fpm::auth::COOKIE_TOKEN,
                        access_token.as_str(),
                    )
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

// it returns identities which matches to given input
pub async fn matched_identities(
    access_token: &str,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    let github_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("github"))
        .collect::<Vec<&fpm::user_group::UserIdentity>>();

    if github_identities.is_empty() {
        return Ok(vec![]);
    }

    let mut matched_identities = vec![];
    // matched_starred_repositories
    matched_identities
        .extend(matched_starred_repos(access_token, github_identities.as_slice()).await?);

    // TODO: matched_team

    Ok(matched_identities)
}

pub async fn matched_starred_repos(
    access_token: &str,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;
    let starred_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-starred") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if starred_repos.is_empty() {
        return Ok(vec![]);
    }
    let user_starred_repos = apis::starred_repo(access_token).await?;
    // filter the user repos with input
    Ok(user_starred_repos
        .into_iter()
        .filter(|user_repo| starred_repos.contains(&user_repo.as_str()))
        .map(|repo| fpm::user_group::UserIdentity {
            key: "github-starred".to_string(),
            value: repo,
        })
        .collect())
}

pub mod apis {

    // TODO: API to starred a repo on behalf of the user
    // API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user

    pub async fn starred_repo(access_token: &str) -> fpm::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user
        // TODO: Handle paginated response

        #[derive(Debug, serde::Deserialize)]
        struct UserRepos {
            full_name: String,
        }
        let starred_repo: Vec<UserRepos> = get_api(
            format!("{}?per_page=100", "https://api.github.com/user/starred").as_str(),
            access_token,
        )
        .await?;
        Ok(starred_repo.into_iter().map(|x| x.full_name).collect())
    }

    pub async fn get_api<T: serde::de::DeserializeOwned>(
        url: &str,
        access_token: &str,
    ) -> fpm::Result<T> {
        let response = reqwest::Client::new()
            .get(url)
            .header(
                reqwest::header::AUTHORIZATION,
                format!("{}{}", "Bearer ", access_token),
            )
            .header(reqwest::header::ACCEPT, "application/json")
            .header(
                reqwest::header::USER_AGENT,
                reqwest::header::HeaderValue::from_static("fpm"),
            )
            .send()
            .await?;

        if !response.status().eq(&reqwest::StatusCode::OK) {
            dbg!(response.text().await?);
            return Err(fpm::Error::APIResponseError(format!(
                "GitHub API ERROR: {}",
                url
            )));
        }

        Ok(response.json().await?)
    }
}

pub mod utils {

    pub fn github_client() -> oauth2::basic::BasicClient {
        use fpm::auth::github::{GITHUB_CLIENT_ID, GITHUB_CLIENT_SECRET};
        oauth2::basic::BasicClient::new(
            GITHUB_CLIENT_ID.to_owned(),
            Some(GITHUB_CLIENT_SECRET.to_owned()),
            oauth2::AuthUrl::new("https://github.com/login/oauth/authorize".to_string()).unwrap(),
            Some(
                oauth2::TokenUrl::new("https://github.com/login/oauth/access_token".to_string())
                    .expect("Invalid token endpoint URL"),
            ),
        )
    }
}
