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
    // https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest
    let client = utils::github_client().set_redirect_uri(oauth2::RedirectUrl::new(redirect_url)?);
    // Note: public_repos user:email all these things are github resources
    // So we have to tell client who is getting logged in what are we going to access
    let (mut authorize_url, _token) = client
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .url();

    // https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest:~:text=an%20appropriate%20display.-,prompt,-OPTIONAL.%20Space%20delimited
    authorize_url
        .query_pairs_mut()
        .append_pair("prompt", "consent");

    //dbg!(&authorize_url);
    // let mut pairs: Vec<(&str, &str)> = vec![("response_type", self.response_type.as_ref())];

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
            //dbg!(&err);
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
    // matched: github-watch
    matched_identities
        .extend(matched_watched_repos(access_token, github_identities.as_slice()).await?);
    // matched: github-follows
    matched_identities
        .extend(matched_followed_org(access_token, github_identities.as_slice()).await?);
    // matched: github-contributor
    matched_identities
        .extend(matched_org_contributors_repos(access_token, github_identities.as_slice()).await?);
    // matched: github-team
    matched_identities
        .extend(matched_org_collaborators_repos(access_token, github_identities.as_slice()).await?);

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

pub async fn matched_watched_repos(
    access_token: &str,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;
    let watched_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-watch") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    if watched_repos.is_empty() {
        return Ok(vec![]);
    }
    let user_watched_repos = apis::watched_repo(access_token).await?;
    // filter the user repos with input
    Ok(user_watched_repos
        .into_iter()
        .filter(|user_repo| watched_repos.contains(&user_repo.as_str()))
        .map(|repo| fpm::user_group::UserIdentity {
            key: "github-watch".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_followed_org(
    access_token: &str,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;
    let followed_orgs = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-follows") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    if followed_orgs.is_empty() {
        return Ok(vec![]);
    }
    let user_followed_orgs = apis::followed_org(access_token).await?;
    // filter the user repos with input
    Ok(user_followed_orgs
        .into_iter()
        .filter(|user_org| followed_orgs.contains(&user_org.as_str()))
        .map(|repo| fpm::user_group::UserIdentity {
            key: "github-follows".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_org_contributors_repos(
    access_token: &str,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut org_repo_contributors: Vec<String> = vec![];
    let contributed_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-contributor") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    let user_name = apis::user_details(access_token).await?;
    if contributed_repos.is_empty() {
        return Ok(vec![]);
    }
    for repo in &contributed_repos {
        let repo_contributors = apis::repo_contributors(access_token, repo).await?;

        if repo_contributors.contains(&user_name) {
            org_repo_contributors.push(String::from(repo.to_owned()));
        }
    }
    // filter the contributor repos with input
    Ok(org_repo_contributors
        .into_iter()
        .filter(|user_repo| contributed_repos.contains(&user_repo.as_str()))
        .map(|repo| fpm::user_group::UserIdentity {
            key: "github-contributor".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_org_collaborators_repos(
    access_token: &str,
    identities: &[&fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut org_repo_collaborator: Vec<String> = vec![];
    let collaborator_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-team") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    let user_name = apis::user_details(access_token).await?;
    if collaborator_repos.is_empty() {
        return Ok(vec![]);
    }
    for repo in &collaborator_repos {
        let repo_collaborator = apis::repo_collaborators(access_token, repo).await?;

        if repo_collaborator.contains(&user_name) {
            dbg!(&access_token);
            org_repo_collaborator.push(String::from(repo.to_owned()));
        }
    }
    // filter the collaborator repos with input
    Ok(org_repo_collaborator
        .into_iter()
        .filter(|user_repo| collaborator_repos.contains(&user_repo.as_str()))
        .map(|repo| fpm::user_group::UserIdentity {
            key: "github-team".to_string(),
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

    pub async fn followed_org(access_token: &str) -> fpm::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/users/followers#list-followers-of-the-authenticated-user
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct UserRepos {
            login: String,
        }
        let watched_repo: Vec<UserRepos> = get_api(
            format!("{}?per_page=100", "https://api.github.com/user/following").as_str(),
            access_token,
        )
        .await?;
        Ok(watched_repo.into_iter().map(|x| x.login).collect())
    }

    pub async fn watched_repo(access_token: &str) -> fpm::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/activity/watching#list-repositories-watched-by-the-authenticated-user
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct UserRepos {
            full_name: String,
        }
        let watched_repo: Vec<UserRepos> = get_api(
            format!(
                "{}?per_page=100",
                "https://api.github.com/user/subscriptions"
            )
            .as_str(),
            access_token,
        )
        .await?;
        Ok(watched_repo.into_iter().map(|x| x.full_name).collect())
    }
    pub async fn repo_contributors(
        access_token: &str,
        repo_name: &str,
    ) -> fpm::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct UserRepos {
            login: String,
        }

        //dbg!(apis::user_details(access_token).await?);
        let repo_contributor: Vec<UserRepos> = get_api(
            format!(
                "{}{}/contributors?per_page=100",
                "https://api.github.com/repos/", repo_name
            )
            .as_str(),
            access_token,
        )
        .await?;
        Ok(repo_contributor.into_iter().map(|x| x.login).collect())
    }
    pub async fn repo_collaborators(
        access_token: &str,
        repo_name: &str,
    ) -> fpm::Result<Vec<String>> {
        // API Docs: https://docs.github.com/en/rest/collaborators/collaborators#list-repository-collaborators
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct UserRepos {
            login: String,
        }

        //dbg!(apis::user_details(access_token).await?);
        let repo_collaborators: Vec<UserRepos> = get_api(
            format!(
                "{}{}/collaborators?per_page=100",
                "https://api.github.com/repos/", repo_name
            )
            .as_str(),
            access_token,
        )
        .await?;
        Ok(repo_collaborators.into_iter().map(|x| x.login).collect())
    }

    pub async fn user_details(access_token: &str) -> fpm::Result<String> {
        // API Docs: https://docs.github.com/en/rest/users/users#get-the-authenticated-user
        // TODO: Handle paginated response
        #[derive(Debug, serde::Deserialize)]
        struct UserDetails {
            login: String,
        }
        let user_obj: UserDetails = get_api("https://api.github.com/user", access_token).await?;

        Ok(String::from(&user_obj.login))
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
            return Err(fpm::Error::APIResponseError(format!(
                "GitHub API ERROR: {}",
                url
            )));
        }

        Ok(response.json().await?)
    }
}

pub mod utils {

    // Lazy means a value which initialize at the first time access
    // we have to access it before using it and make sure to use it while starting a server
    // TODO: they should be configured with auth feature flag
    // if feature flag auth is enabled Make sure that before accessing in the API these variable
    // are set
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

    pub fn github_client() -> oauth2::basic::BasicClient {
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
