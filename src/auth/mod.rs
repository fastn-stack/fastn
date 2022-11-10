pub(crate) mod config;
pub(crate) mod discord;
pub(crate) mod github;
pub(crate) mod gmail;
pub(crate) mod routes;
pub(crate) mod slack;
pub(crate) mod telegram;
pub mod utils;

pub const COOKIE_TOKEN: &str = "access_token";

// TODO: rename the method later
// bridge between fpm to auth to check
pub async fn get_auth_identities(
    cookies: &std::collections::HashMap<String, String>,
    identities: &[fpm::user_group::UserIdentity],
) -> fpm::Result<Vec<fpm::user_group::UserIdentity>> {
    dbg!(&cookies);

    let access_token = cookies.get(COOKIE_TOKEN).ok_or_else(|| {
        fpm::Error::GenericError("access_token not found in the cookies".to_string())
    })?;

    // TODO: which API to from which platform based on identity
    // identity can be github-*, discord-*, and etc...
    let matched_identities = github::matched_identities(access_token.as_str(), identities).await?;

    //TODO: Call discord::matched_identities
    //TODO: Call google::matched_identities
    //TODO: Call twitter::matched_identities
    dbg!(&matched_identities);
    Ok(matched_identities)
}
