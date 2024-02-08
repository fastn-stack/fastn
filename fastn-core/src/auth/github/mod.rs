mod apis;
mod utils;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub access_token: String,
    pub user: fastn_core::auth::FastnUser,
}

pub async fn login(
    ds: &fastn_ds::DocumentStore,
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
    let (authorize_url, _token) = fastn_core::auth::github::utils::github_client(ds)
        .await
        .unwrap()
        .set_redirect_uri(oauth2::RedirectUrl::new(redirect_url)?)
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .add_scope(oauth2::Scope::new("read:org".to_string()))
        .url();

    Ok(fastn_core::http::redirect(authorize_url.to_string()))
}

// route: /-/auth/github/done/
// In this API we are accessing
// the token and setting it to cookies
pub async fn callback(
    req: &fastn_core::http::Request,
    ds: &fastn_ds::DocumentStore,
    db_pool: &fastn_core::db::PgPool,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    use diesel::prelude::*;
    use diesel_async::RunQueryDsl;

    let code = req.q("code", "".to_string())?;
    // TODO: CSRF check

    // TODO: if a user is already logged in using emailpassword and uses github oauth
    // present a merge account option:
    // ask to add github email to the logged in user's profile
    // ask to update details by giving a form
    // redirect to next for now
    if fastn_core::auth::utils::is_authenticated(req) {
        return Ok(actix_web::HttpResponse::Found()
            .append_header((actix_web::http::header::LOCATION, next))
            .finish());
    }

    let access_token = match fastn_core::auth::github::utils::github_client(ds)
        .await
        .unwrap()
        .exchange_code(oauth2::AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(access_token) => oauth2::TokenResponse::access_token(&access_token)
            .secret()
            .to_string(),
        Err(e) => return Ok(fastn_core::server_error!("{}", e.to_string())),
    };

    let mut gh_user = fastn_core::auth::github::apis::user_details(access_token.as_str()).await?;

    if gh_user.email.is_none() {
        // pick primary email
        let emails = fastn_core::auth::github::apis::user_emails(access_token.as_str()).await?;
        let primary = emails
            .into_iter()
            .find(|e| e.primary)
            .expect("primary email must exist for a github account");

        gh_user.email = Some(primary.email);
    }

    let mut conn = db_pool
        .get()
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to get connection to db. {:?}", e),
        })?;

    let existing_email_and_user_id: Option<(fastn_core::utils::CiString, i32)> =
        fastn_core::schema::fastn_user_email::table
            .select((
                fastn_core::schema::fastn_user_email::email,
                fastn_core::schema::fastn_user_email::user_id,
            ))
            .filter(
                fastn_core::schema::fastn_user_email::email.eq(fastn_core::utils::citext(
                    gh_user
                        .email
                        .as_ref()
                        .expect("Every github account has a primary email"),
                )),
            )
            .first(&mut conn)
            .await
            .optional()?;

    if existing_email_and_user_id.is_some() {
        // user already exists, just create a session and redirect to next
        let (_, user_id) = existing_email_and_user_id.unwrap();

        let session_id: i32 = diesel::insert_into(fastn_core::schema::fastn_session::table)
            .values(fastn_core::schema::fastn_session::user_id.eq(&user_id))
            .returning(fastn_core::schema::fastn_session::id)
            .get_result(&mut conn)
            .await?;

        tracing::info!("session created. session_id: {}", &session_id);

        // TODO: access_token expires?
        // handle refresh tokens
        let token_id: i32 = diesel::insert_into(fastn_core::schema::fastn_oauthtoken::table)
            .values((
                fastn_core::schema::fastn_oauthtoken::session_id.eq(session_id),
                fastn_core::schema::fastn_oauthtoken::token.eq(access_token),
                fastn_core::schema::fastn_oauthtoken::provider.eq("github"),
            ))
            .returning(fastn_core::schema::fastn_oauthtoken::id)
            .get_result(&mut conn)
            .await?;

        tracing::info!("token stored. token_id: {}", &token_id);

        return fastn_core::auth::set_session_cookie_and_redirect_to_next(
            req, ds, session_id, next,
        )
        .await;
    }

    // first time login, create fastn_user
    let user = diesel::insert_into(fastn_core::schema::fastn_user::table)
        .values((
            fastn_core::schema::fastn_user::username.eq(gh_user.login),
            fastn_core::schema::fastn_user::password.eq(""),
            // TODO: should present an onabording form that asks for a name if github name is null
            fastn_core::schema::fastn_user::name.eq(gh_user.name.unwrap_or_default()),
        ))
        .returning(fastn_core::auth::FastnUser::as_returning())
        .get_result(&mut conn)
        .await?;

    tracing::info!("fastn_user created. user_id: {:?}", &user.id);

    let email_id: i32 = diesel::insert_into(fastn_core::schema::fastn_user_email::table)
        .values((
            fastn_core::schema::fastn_user_email::user_id.eq(&user.id),
            fastn_core::schema::fastn_user_email::email.eq(fastn_core::utils::citext(
                gh_user
                    .email
                    .as_ref()
                    .expect("Every github account has a primary email"),
            )),
            fastn_core::schema::fastn_user_email::verified.eq(true),
            fastn_core::schema::fastn_user_email::primary.eq(true),
        ))
        .returning(fastn_core::schema::fastn_user_email::id)
        .get_result(&mut conn)
        .await?;

    tracing::info!("fastn_user_email created. email: {:?}", &email_id);

    // TODO: session should store device that was used to login (chrome desktop on windows)
    let session_id: i32 = diesel::insert_into(fastn_core::schema::fastn_session::table)
        .values((fastn_core::schema::fastn_session::user_id.eq(&user.id),))
        .returning(fastn_core::schema::fastn_session::id)
        .get_result(&mut conn)
        .await?;

    tracing::info!("session created. session_id: {}", &session_id);

    // TODO: access_token expires?
    // handle refresh tokens
    let token_id: i32 = diesel::insert_into(fastn_core::schema::fastn_oauthtoken::table)
        .values((
            fastn_core::schema::fastn_oauthtoken::session_id.eq(session_id),
            fastn_core::schema::fastn_oauthtoken::token.eq(access_token),
            fastn_core::schema::fastn_oauthtoken::provider.eq("github"),
        ))
        .returning(fastn_core::schema::fastn_oauthtoken::id)
        .get_result(&mut conn)
        .await?;

    tracing::info!("token stored. token_id: {}", &token_id);

    // Onboarding step is opt-in
    let onboarding_enabled = ds.env("FASTN_AUTH_ADD_ONBOARDING_STEP").await.is_ok();

    let next_path = if onboarding_enabled {
        format!("/-/auth/onboarding/?next={}", next)
    } else {
        next.to_string()
    };

    // redirect to onboarding route with a GET request
    let mut resp =
        fastn_core::auth::set_session_cookie_and_redirect_to_next(req, ds, session_id, next_path)
            .await?;

    if onboarding_enabled {
        resp.add_cookie(
            &actix_web::cookie::Cookie::build(
                fastn_core::auth::FIRST_TIME_SESSION_COOKIE_NAME,
                "1",
            )
            .domain(fastn_core::auth::utils::domain(req.connection_info.host()))
            .path("/")
            .finish(),
        )
        .map_err(|e| fastn_core::Error::generic(format!("failed to set cookie: {e}")))?;
    }

    Ok(resp)
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

        if repo_contributors.contains(&ud.user.username) {
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

        if repo_collaborator.contains(&ud.user.username) {
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
            if team_members.contains(&ud.user.username) {
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
            ud.user.username.as_str(),
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
