pub(crate) mod config;
pub(crate) mod discord;
pub(crate) mod github;
pub(crate) mod gmail;
pub(crate) mod routes;
pub(crate) mod slack;
pub(crate) mod telegram;
pub mod utils;

// TODO: rename the method later
// ridge between fpm to auth to check
pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    dbg!(&cookies);

    let access_token = cookies.get("access_token").ok_or_else(|| {
        fpm::Error::GenericError("access_token not found in the cookies".to_string())
    })?;

    // TODO: which API to from which platform based on identity
    // identity can be github-*, discord-*, and etc...
    let user_starred_repos = github::get_starred_repo(access_token.as_str(), identities).await?;

    dbg!(&user_starred_repos);

    Ok(user_starred_repos
        .into_iter()
        .map(|repo| fpm::user_group::UserIdentity {
            key: "github-starred".to_string(),
            value: repo,
        })
        .collect())
}
