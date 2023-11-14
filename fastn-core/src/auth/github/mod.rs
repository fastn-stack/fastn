pub mod apis;
mod utils;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub access_token: String,
    pub username: String,
}

pub async fn login(
    req: &fastn_core::http::Request,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    let redirect_url: String = format!(
        "{}://{}/-/auth/github/?next={}",
        req.connection_info.scheme(),
        req.connection_info.host(),
        next, // TODO: we should url escape this
    );

    // Note: public_repos user:email all these things are github resources
    // So we have to tell oauth_client who is getting logged in what are we going to access
    let (mut authorize_url, _token) = fastn_core::auth::github::utils::github_client()
        .set_redirect_uri(oauth2::RedirectUrl::new(redirect_url)?)
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .add_scope(oauth2::Scope::new("read:org".to_string()))
        .url();

    // https://openid.net/specs/openid-connect-core-1_0.html#AuthRequest:~:text=an%20appropriate%20display.-,prompt,-OPTIONAL.%20Space%20delimited
    authorize_url
        .query_pairs_mut()
        .append_pair("prompt", "consent");

    Ok(fastn_core::http::redirect(authorize_url.to_string()))
}

// route: /-/auth/github/done/
// In this API we are accessing
// the token and setting it to cookies
pub async fn callback(
    req: &fastn_core::http::Request,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    let code = req.q("code", "".to_string())?;
    // TODO: CSRF check

    let access_token = match fastn_core::auth::github::utils::github_client()
        .exchange_code(oauth2::AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(access_token) => oauth2::TokenResponse::access_token(&access_token)
            .secret()
            .to_string(),
        Err(e) => return Ok(fastn_core::server_error!("{}", e.to_string())),
    };

    let gh_user = fastn_core::auth::github::apis::user_details(access_token.as_str()).await?;

    let ud = UserDetail {
        username: gh_user.login,
        access_token,
    };

    let user_detail_str = serde_json::to_string(&ud)?;
    return Ok(actix_web::HttpResponse::Found()
        .cookie(
            actix_web::cookie::Cookie::build(
                fastn_core::auth::AuthProviders::GitHub.as_str(),
                fastn_core::auth::utils::encrypt_str(&user_detail_str).await,
            )
            .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
            .path("/")
            .permanent()
            // TODO: AbrarK is running on http,
            // will remove it later
            // .secure(true)
            .finish(),
        )
        .append_header((actix_web::http::header::LOCATION, next))
        .finish());
}

// it returns identities which matches to given input
pub async fn matched_identities(
    ud: UserDetail,
    identities: &[fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    let github_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("github"))
        .collect::<Vec<&fastn_core::user_group::UserIdentity>>();

    if github_identities.is_empty() {
        return Ok(vec![]);
    }

    let mut matched_identities = vec![];
    // matched_starred_repositories
    matched_identities.extend(matched_starred_repos(&ud, github_identities.as_slice()).await?);
    // matched: github-watches
    matched_identities.extend(matched_watched_repos(&ud, github_identities.as_slice()).await?);
    // matched: github-follows
    matched_identities.extend(matched_followed_org(&ud, github_identities.as_slice()).await?);
    // matched: github-contributor
    matched_identities.extend(matched_contributed_repos(&ud, github_identities.as_slice()).await?);
    // matched: github-collaborator
    matched_identities.extend(matched_collaborated_repos(&ud, github_identities.as_slice()).await?);
    // matched: github-team
    matched_identities.extend(matched_org_teams(&ud, github_identities.as_slice()).await?);
    // matched: github-sponsor
    matched_identities.extend(matched_sponsored_org(&ud, github_identities.as_slice()).await?);

    Ok(matched_identities)
}

pub async fn matched_starred_repos(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
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
    let user_starred_repos =
        fastn_core::auth::github::apis::starred_repo(ud.access_token.as_str()).await?;
    // filter the user starred repos with input
    Ok(user_starred_repos
        .into_iter()
        .filter(|user_repo| starred_repos.contains(&user_repo.as_str()))
        .map(|repo| fastn_core::user_group::UserIdentity {
            key: "github-starred".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_watched_repos(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let watched_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-watches") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();
    if watched_repos.is_empty() {
        return Ok(vec![]);
    }
    let user_watched_repos =
        fastn_core::auth::github::apis::watched_repo(ud.access_token.as_str()).await?;
    // filter the user watched repos with input
    Ok(user_watched_repos
        .into_iter()
        .filter(|user_repo| watched_repos.contains(&user_repo.as_str()))
        .map(|repo| fastn_core::user_group::UserIdentity {
            key: "github-watches".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_followed_org(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
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
    let user_followed_orgs =
        fastn_core::auth::github::apis::followed_org(ud.access_token.as_str()).await?;
    // filter the user followed orgs with input
    Ok(user_followed_orgs
        .into_iter()
        .filter(|user_org| followed_orgs.contains(&user_org.as_str()))
        .map(|org| fastn_core::user_group::UserIdentity {
            key: "github-follows".to_string(),
            value: org,
        })
        .collect())
}

pub async fn matched_contributed_repos(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_repo_contributors_list: Vec<String> = vec![];
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

    if contributed_repos.is_empty() {
        return Ok(vec![]);
    }
    for repo in &contributed_repos {
        let repo_contributors =
            fastn_core::auth::github::apis::repo_contributors(ud.access_token.as_str(), repo)
                .await?;

        if repo_contributors.contains(&ud.username) {
            matched_repo_contributors_list.push(String::from(repo.to_owned()));
        }
    }
    // filter the user contributed repos with input
    Ok(matched_repo_contributors_list
        .into_iter()
        .filter(|user_repo| contributed_repos.contains(&user_repo.as_str()))
        .map(|repo| fastn_core::user_group::UserIdentity {
            key: "github-contributor".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_collaborated_repos(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_repo_collaborator_list: Vec<String> = vec![];
    let collaborated_repos = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-collaborator") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if collaborated_repos.is_empty() {
        return Ok(vec![]);
    }
    for repo in &collaborated_repos {
        let repo_collaborator =
            fastn_core::auth::github::apis::repo_collaborators(ud.access_token.as_str(), repo)
                .await?;

        if repo_collaborator.contains(&ud.username) {
            matched_repo_collaborator_list.push(String::from(repo.to_owned()));
        }
    }
    // filter the user collaborated repos with input
    Ok(matched_repo_collaborator_list
        .into_iter()
        .filter(|user_repo| collaborated_repos.contains(&user_repo.as_str()))
        .map(|repo| fastn_core::user_group::UserIdentity {
            key: "github-collaborator".to_string(),
            value: repo,
        })
        .collect())
}

pub async fn matched_org_teams(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut matched_org_teams: Vec<String> = vec![];
    let org_teams = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-team") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if org_teams.is_empty() {
        return Ok(vec![]);
    }

    for org_team in org_teams.iter() {
        if let Some((org_name, team_name)) = org_team.split_once('/') {
            let team_members: Vec<String> = fastn_core::auth::github::apis::team_members(
                ud.access_token.as_str(),
                org_name,
                team_name,
            )
            .await?;
            if team_members.contains(&ud.username) {
                matched_org_teams.push(org_team.to_string());
            }
        }
        // TODO:
        // Return Error if org-name/team-name does not come
    }
    // filter the user joined teams with input
    Ok(matched_org_teams
        .into_iter()
        .map(|org_team| fastn_core::user_group::UserIdentity {
            key: "github-team".to_string(),
            value: org_team,
        })
        .collect())
}

pub async fn matched_sponsored_org(
    ud: &UserDetail,
    identities: &[&fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    use itertools::Itertools;
    let mut sponsored_users_list: Vec<String> = vec![];

    let sponsors_list = identities
        .iter()
        .filter_map(|i| {
            if i.key.eq("github-sponsor") {
                Some(i.value.as_str())
            } else {
                None
            }
        })
        .collect_vec();

    if sponsors_list.is_empty() {
        return Ok(vec![]);
    }

    for sponsor in sponsors_list.iter() {
        if fastn_core::auth::github::apis::is_user_sponsored(
            ud.access_token.as_str(),
            ud.username.as_str(),
            sponsor.to_owned(),
        )
        .await?
        {
            sponsored_users_list.push(sponsor.to_string());
        }
    }
    // return the sponsor list
    Ok(sponsored_users_list
        .into_iter()
        .map(|sponsor| fastn_core::user_group::UserIdentity {
            key: "github-sponsor".to_string(),
            value: sponsor,
        })
        .collect())
}
