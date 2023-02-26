#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub token: String,
    pub user_name: String,
    pub user_id: String,
}
pub async fn matched_identities(
    _ud: UserDetail,
    _identities: &[fastn_core::user_group::UserIdentity],
) -> fastn_core::Result<Vec<fastn_core::user_group::UserIdentity>> {
    /*let bitbucket_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("bitbucket"))
        .collect::<Vec<&fastn_core::user_group::UserIdentity>>();

    if bitbucket_identities.is_empty() {
        return Ok(vec![]);
    }*/

    let matched_identities = vec![];

    Ok(matched_identities)
}
