#[derive(Debug, serde::Deserialize)]
pub struct GraphQLResp {
    pub data: Data,
}

#[derive(Debug, serde::Deserialize)]
pub struct Data {
    pub user: User,
}

#[derive(Debug, serde::Deserialize)]
pub struct User {
    #[serde(rename = "isSponsoredBy")]
    pub is_sponsored_by: bool,
}

// TODO: API to starred a repo on behalf of the user
// API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user
pub async fn starred_repo(token: &str) -> fastn_core::Result<Vec<String>> {
    // API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user
    // TODO: Handle paginated response

    #[derive(Debug, serde::Deserialize)]
    struct UserRepos {
        full_name: String,
    }
    let starred_repo: Vec<UserRepos> =
        fastn_core::http::get_api("https://api.github.com/user/starred?per_page=100", token)
            .await?;
    Ok(starred_repo.into_iter().map(|x| x.full_name).collect())
}

pub async fn followed_org(token: &str) -> fastn_core::Result<Vec<String>> {
    // API Docs: https://docs.github.com/en/rest/users/followers#list-followers-of-the-authenticated-user
    // TODO: Handle paginated response
    #[derive(Debug, serde::Deserialize)]
    struct FollowedOrg {
        login: String,
    }
    let watched_repo: Vec<FollowedOrg> =
        fastn_core::http::get_api("https://api.github.com/user/following?per_page=100", token)
            .await?;
    Ok(watched_repo.into_iter().map(|x| x.login).collect())
}

pub async fn team_members(
    token: &str,
    org_title: &str,
    team_slug: &str,
) -> fastn_core::Result<Vec<String>> {
    // API Docs: https://docs.github.com/en/rest/teams/members#list-team-members
    // TODO: Handle paginated response
    #[derive(Debug, serde::Deserialize)]
    struct TeamMembers {
        login: String,
    }

    let user_orgs: Vec<TeamMembers> = fastn_core::http::get_api(
        format!("https://api.github.com/orgs/{org_title}/teams/{team_slug}/members?per_page=100",),
        token,
    )
    .await?;
    Ok(user_orgs.into_iter().map(|x| x.login).collect())
}

pub async fn watched_repo(token: &str) -> fastn_core::Result<Vec<String>> {
    // API Docs: https://docs.github.com/en/rest/activity/watching#list-repositories-watched-by-the-authenticated-user
    // TODO: Handle paginated response
    #[derive(Debug, serde::Deserialize)]
    struct UserRepos {
        full_name: String,
    }
    let watched_repo: Vec<UserRepos> = fastn_core::http::get_api(
        "https://api.github.com/user/subscriptions?per_page=100",
        token,
    )
    .await?;
    Ok(watched_repo.into_iter().map(|x| x.full_name).collect())
}

pub async fn repo_contributors(token: &str, repo_name: &str) -> fastn_core::Result<Vec<String>> {
    // API Docs: https://docs.github.com/en/rest/activity/starring#list-repositories-starred-by-the-authenticated-user
    // TODO: Handle paginated response
    #[derive(Debug, serde::Deserialize)]
    struct RepoContributor {
        login: String,
    }
    let repo_contributor: Vec<RepoContributor> = fastn_core::http::get_api(
        format!("https://api.github.com/repos/{repo_name}/contributors?per_page=100",),
        token,
    )
    .await?;
    Ok(repo_contributor.into_iter().map(|x| x.login).collect())
}

pub async fn repo_collaborators(token: &str, repo_name: &str) -> fastn_core::Result<Vec<String>> {
    // API Docs: https://docs.github.com/en/rest/collaborators/collaborators#list-repository-collaborators
    // TODO: Handle paginated response
    #[derive(Debug, serde::Deserialize)]
    struct RepoCollaborator {
        login: String,
    }
    let repo_collaborators_list: Vec<RepoCollaborator> = fastn_core::http::get_api(
        format!("https://api.github.com/repos/{repo_name}/collaborators?per_page=100"),
        token,
    )
    .await?;
    Ok(repo_collaborators_list
        .into_iter()
        .map(|x| x.login)
        .collect())
}

pub async fn is_user_sponsored(
    token: &str,
    username: &str,
    sponsored_by: &str,
) -> fastn_core::Result<bool> {
    let query = format!(
        r#"query {{ 
                user(login: "{username}") 
                {{ isSponsoredBy(accountLogin: "{sponsored_by}" )}} 
            }}"#
    );
    let sponsor_obj: GraphQLResp = fastn_core::http::github_graphql(query.as_str(), token).await?;
    if sponsor_obj.data.user.is_sponsored_by {
        Ok(true)
    } else {
        Ok(false)
    }
}

pub async fn user_details(access_token: &str) -> fastn_core::Result<fastn_core::auth::FastnUser> {
    // API Docs: https://docs.github.com/en/rest/users/users#get-the-authenticated-user
    // TODO: Handle paginated response
    let user_obj: fastn_core::auth::FastnUser =
        fastn_core::http::get_api("https://api.github.com/user", access_token).await?;

    Ok(user_obj)
}

#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct GhEmail {
    pub email: String,
    pub verified: bool,
    pub primary: bool,
    pub visibility: String,
}

pub async fn user_emails(access_token: &str) -> fastn_core::Result<Vec<GhEmail>> {
    // API Docs: https://docs.github.com/en/rest/users/emails?apiVersion=2022-11-28#list-email-addresses-for-the-authenticated-user
    let user_obj: Vec<GhEmail> =
        fastn_core::http::get_api("https://api.github.com/user/emails", access_token).await?;

    Ok(user_obj)
}
