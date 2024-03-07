mod apis;
mod utils;

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub access_token: String,
    pub user: fastn_core::auth::FastnUser,
}

pub async fn login(
    req: &fastn_core::http::Request,
    ds: &fastn_ds::DocumentStore,
    next: String,
) -> fastn_core::Result<fastn_core::http::Response> {
    // [INFO] logging: github-login
    req.log(
        "github-login",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    let redirect_url: String = format!(
        "{scheme}://{host}{callback_url}?next={next}",
        scheme = req.connection_info.scheme(),
        host = req.connection_info.host(),
        callback_url = fastn_core::auth::Route::GithubCallback,
        next = next, // TODO: we should url escape this
    );

    // Note: public_repos user:email all these things are github resources
    // So we have to tell oauth_client who is getting logged in what are we going to access
    let github_client = match fastn_core::auth::github::utils::github_client(ds).await {
        Ok(client) => client,
        Err(e) => {
            // [ERROR] logging (Environment Error)
            let log_err_message = format!("environment: {:?}", &e);
            req.log(
                "github-login",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    let (authorize_url, _token) = github_client
        .set_redirect_uri(oauth2::RedirectUrl::new(redirect_url)?)
        .authorize_url(oauth2::CsrfToken::new_random)
        .add_scope(oauth2::Scope::new("public_repo".to_string()))
        .add_scope(oauth2::Scope::new("user:email".to_string()))
        .add_scope(oauth2::Scope::new("read:org".to_string()))
        .url();

    Ok(fastn_core::http::temporary_redirect(
        authorize_url.to_string(),
    ))
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

    // [INFO] logging: github-callback
    req.log(
        "github-callback",
        fastn_core::log::OutcomeKind::Info,
        file!(),
        line!(),
    );

    let now = chrono::Utc::now();
    let code = req.q("code", "".to_string())?;
    // TODO: CSRF check

    // TODO: if a user is already logged in using emailpassword and uses github oauth
    // present a merge account option:
    // ask to add github email to the logged in user's profile
    // ask to update details by giving a form
    // redirect to next for now
    if fastn_core::auth::utils::is_authenticated(req) {
        // [SUCCESS] logging: already authenticated
        let log_success_message = "user: already authenticated".to_string();
        req.log(
            "github-callback",
            fastn_core::log::OutcomeKind::Success(fastn_core::log::Outcome::Descriptive(
                log_success_message,
            )),
            file!(),
            line!(),
        );

        return Ok(fastn_core::http::temporary_redirect(next));
    }

    let client = match fastn_core::auth::github::utils::github_client(ds).await {
        Ok(client) => client,
        Err(e) => {
            // [ERROR] logging (Environment Error)
            let log_err_message = format!("environment: {:?}", &e);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e.into());
        }
    };

    let access_token = match client
        .exchange_code(oauth2::AuthorizationCode::new(code))
        .request_async(oauth2::reqwest::async_http_client)
        .await
    {
        Ok(access_token) => oauth2::TokenResponse::access_token(&access_token)
            .secret()
            .to_string(),
        Err(e) => {
            // [ERROR] logging (Request Token Error)
            let log_err_message = format!("request token: {:?}", &e);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Ok(fastn_core::server_error!("{}", e.to_string()));
        }
    };

    let mut gh_user =
        match fastn_core::auth::github::apis::user_details(access_token.as_str()).await {
            Ok(user) => user,
            Err(e) => {
                // [ERROR] logging (Github user Error)
                let log_err_message = format!("github user: {:?}", &e);
                req.log(
                    "github-callback",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                return Err(e);
            }
        };

    if gh_user.email.is_none() {
        // pick primary email
        let emails = match fastn_core::auth::github::apis::user_emails(access_token.as_str()).await
        {
            Ok(emails) => emails,
            Err(e) => {
                // [ERROR] logging (Github email Error)
                let log_err_message = format!("github email: {:?}", &e);
                req.log(
                    "github-callback",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                return Err(e);
            }
        };

        let primary = match emails.into_iter().find(|e| e.primary) {
            Some(primary) => primary,
            None => {
                // [ERROR] logging (Github primary email Error)
                let err_message = "primary email must exist for a github account".to_string();
                let log_err_message = format!("github primary email: {:?}", &err_message);
                req.log(
                    "github-callback",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                return Err(fastn_core::Error::NotFound(err_message));
            }
        };

        gh_user.email = Some(primary.email);
    }

    let mut conn = match db_pool.get().await {
        Ok(conn) => conn,
        Err(e) => {
            // [ERROR] logging (Database Error)
            let err_message = format!("Failed to get connection to db. {:?}", &e);
            let log_err_message = format!("database: {:?}", &err_message);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    let existing_email_and_user_id: Option<(fastn_core::utils::CiString, i64)> =
        match fastn_core::schema::fastn_user_email::table
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
            .optional()
        {
            Ok(v) => v,
            Err(e) => {
                // [ERROR] logging (Database Error)
                let err_message = format!("{:?}", &e);
                let log_err_message = format!("database: {:?}", &err_message);
                req.log(
                    "github-callback",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                return Err(fastn_core::Error::DatabaseError {
                    message: err_message,
                });
            }
        };

    if let Some((_, user_id)) = existing_email_and_user_id {
        // user already exists, just create a session and redirect to next

        let session_id: i64 =
            match diesel::insert_into(fastn_core::schema::fastn_auth_session::table)
                .values((
                    fastn_core::schema::fastn_auth_session::user_id.eq(&user_id),
                    fastn_core::schema::fastn_auth_session::created_at.eq(now),
                    fastn_core::schema::fastn_auth_session::updated_at.eq(now),
                ))
                .returning(fastn_core::schema::fastn_auth_session::id)
                .get_result(&mut conn)
                .await
            {
                Ok(id) => id,
                Err(e) => {
                    // [ERROR] logging (Database Error)
                    let err_message = format!("{:?}", &e);
                    let log_err_message = format!("database: {:?}", &err_message);
                    req.log(
                        "github-callback",
                        fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                        file!(),
                        line!(),
                    );
                    return Err(fastn_core::Error::DatabaseError {
                        message: err_message,
                    });
                }
            };

        tracing::info!("session created. session_id: {}", &session_id);

        // TODO: access_token expires?
        // handle refresh tokens
        let token_id: i64 = match diesel::insert_into(fastn_core::schema::fastn_oauthtoken::table)
            .values((
                fastn_core::schema::fastn_oauthtoken::session_id.eq(session_id),
                fastn_core::schema::fastn_oauthtoken::token.eq(access_token),
                fastn_core::schema::fastn_oauthtoken::provider.eq("github"),
                fastn_core::schema::fastn_oauthtoken::created_at.eq(now),
                fastn_core::schema::fastn_oauthtoken::updated_at.eq(now),
            ))
            .returning(fastn_core::schema::fastn_oauthtoken::id)
            .get_result(&mut conn)
            .await
        {
            Ok(id) => id,
            Err(e) => {
                // [ERROR] logging (Database Error)
                let err_message = format!("{:?}", &e);
                let log_err_message = format!("database: {:?}", &err_message);
                req.log(
                    "github-callback",
                    fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                    file!(),
                    line!(),
                );
                return Err(fastn_core::Error::DatabaseError {
                    message: err_message,
                });
            }
        };

        tracing::info!("token stored. token_id: {}", &token_id);

        return fastn_core::auth::set_session_cookie_and_redirect_to_next(
            req,
            "github-callback",
            ds,
            session_id,
            next,
        )
        .await;
    }

    // first time login, create fastn_user
    let user = match diesel::insert_into(fastn_core::schema::fastn_user::table)
        .values((
            fastn_core::schema::fastn_user::username.eq(gh_user.login),
            fastn_core::schema::fastn_user::password.eq(""),
            // TODO: should present an onboarding form that asks for a name if github name is null
            fastn_core::schema::fastn_user::name.eq(gh_user.name.unwrap_or_default()),
            fastn_core::schema::fastn_user::verified_email.eq(true),
            fastn_core::schema::fastn_user::email.eq(fastn_core::utils::citext(
                gh_user
                    .email
                    .as_ref()
                    .expect("Every github account has a primary email"),
            )),
            fastn_core::schema::fastn_user::created_at.eq(now),
            fastn_core::schema::fastn_user::updated_at.eq(now),
        ))
        .returning(fastn_core::auth::FastnUser::as_returning())
        .get_result(&mut conn)
        .await
    {
        Ok(fastn_user) => fastn_user,
        Err(e) => {
            // [ERROR] logging (Database Error)
            let err_message = format!("{:?}", &e);
            let log_err_message = format!("database: {:?}", &err_message);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    tracing::info!("fastn_user created. user_id: {:?}", &user.id);

    let email_id: i64 = match diesel::insert_into(fastn_core::schema::fastn_user_email::table)
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
            fastn_core::schema::fastn_user_email::created_at.eq(now),
            fastn_core::schema::fastn_user_email::updated_at.eq(now),
        ))
        .returning(fastn_core::schema::fastn_user_email::id)
        .get_result(&mut conn)
        .await
    {
        Ok(email_id) => email_id,
        Err(e) => {
            // [ERROR] logging (Database Error)
            let err_message = format!("{:?}", &e);
            let log_err_message = format!("database: {:?}", &err_message);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    tracing::info!("fastn_user_email created. email: {:?}", &email_id);

    // TODO: session should store device that was used to login (chrome desktop on windows)
    let session_id: i64 = match diesel::insert_into(fastn_core::schema::fastn_auth_session::table)
        .values((
            fastn_core::schema::fastn_auth_session::user_id.eq(&user.id),
            fastn_core::schema::fastn_auth_session::created_at.eq(now),
            fastn_core::schema::fastn_auth_session::updated_at.eq(now),
        ))
        .returning(fastn_core::schema::fastn_auth_session::id)
        .get_result(&mut conn)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            // [ERROR] logging (Database Error)
            let err_message = format!("{:?}", &e);
            let log_err_message = format!("database: {:?}", &err_message);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    tracing::info!("session created. session_id: {}", &session_id);

    // TODO: access_token expires?
    // handle refresh tokens
    let token_id: i64 = match diesel::insert_into(fastn_core::schema::fastn_oauthtoken::table)
        .values((
            fastn_core::schema::fastn_oauthtoken::session_id.eq(session_id),
            fastn_core::schema::fastn_oauthtoken::token.eq(access_token),
            fastn_core::schema::fastn_oauthtoken::provider.eq("github"),
            fastn_core::schema::fastn_oauthtoken::created_at.eq(now),
            fastn_core::schema::fastn_oauthtoken::updated_at.eq(now),
        ))
        .returning(fastn_core::schema::fastn_oauthtoken::id)
        .get_result(&mut conn)
        .await
    {
        Ok(id) => id,
        Err(e) => {
            // [ERROR] logging (Database Error)
            let err_message = format!("{:?}", &e);
            let log_err_message = format!("database: {:?}", &err_message);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(fastn_core::Error::DatabaseError {
                message: err_message,
            });
        }
    };

    tracing::info!("token stored. token_id: {}", &token_id);

    // Onboarding step is opt-in
    let onboarding_enabled = ds.env("FASTN_AUTH_ADD_ONBOARDING_STEP").await.is_ok();

    let next_path = if onboarding_enabled {
        format!(
            "{onboarding_route}?next={next}",
            onboarding_route = fastn_core::auth::Route::Onboarding
        )
    } else {
        next.to_string()
    };

    // redirect to onboarding route with a GET request
    let mut resp = match fastn_core::auth::set_session_cookie_and_redirect_to_next(
        req,
        "github-callback",
        ds,
        session_id,
        next_path,
    )
    .await
    {
        Ok(response) => response,
        Err(e) => {
            // [ERROR] logging (Session Cookie Error)
            let err_message = format!("{:?}", &e);
            let log_err_message = format!("session cookie: {:?}", &err_message);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            return Err(e);
        }
    };

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
        .map_err(|e| {
            // [ERROR] logging (Set Cookie Error)
            let err_message = format!("failed to set cookie: {:?}", &e);
            let log_err_message = format!("set cookie: {:?}", &err_message);
            req.log(
                "github-callback",
                fastn_core::log::OutcomeKind::error_descriptive(log_err_message),
                file!(),
                line!(),
            );
            fastn_core::Error::generic(err_message)
        })?;
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
