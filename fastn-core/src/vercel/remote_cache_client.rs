pub(crate) struct RemoteClient {
    token: String,
    team_id: Option<String>,
    user_agent: String,
}

impl RemoteClient {
    fn new(token: String, team_id: Option<String>, product: String) -> RemoteClient {
        RemoteClient {
            token,
            team_id,
            user_agent: super::utils::get_user_agent(product.as_str()),
        }
    }

    fn get_endpoint_url(self, hash: String) -> fastn_core::Result<String> {
        if hash.contains('/') {
            return Err(fastn_core::error::Error::GenericError(
                "Invalid hash: Cannot contain '/'".to_string(),
            ));
        }
        let params = if let Some(team_id) = &self.team_id {
            format!("?teamId={}", team_id)
        } else {
            "".to_string()
        };
        Ok(format!(
            "{}/{}{}",
            super::constants::REMOTE_CACHE_ENDPOINT,
            hash,
            params
        ))
    }
}
