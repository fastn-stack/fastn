#[derive(Debug, serde::Deserialize, serde::Serialize)]
pub struct UserDetail {
    pub token: String,
    pub user_name: String,
    pub user_id: String,
}
pub async fn matched_identities(
    _ud: UserDetail,
    _identities: &[fastn::user_group::UserIdentity],
) -> fastn::Result<Vec<fastn::user_group::UserIdentity>> {
    /*let gmail_identities = identities
        .iter()
        .filter(|identity| identity.key.starts_with("gmail"))
        .collect::<Vec<&fastn::user_group::UserIdentity>>();

    if gmail_identities.is_empty() {
        return Ok(vec![]);
    }*/

    let matched_identities = vec![];

    Ok(matched_identities)
}
