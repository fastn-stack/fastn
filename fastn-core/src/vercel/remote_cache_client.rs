#[derive(Debug)]
pub(crate) struct RemoteClient {
    token: String,
    team_id: Option<String>,
    user_agent: String,
}

#[allow(dead_code)]
impl RemoteClient {
    pub fn new(token: String, team_id: Option<String>, product: String) -> RemoteClient {
        RemoteClient {
            token,
            team_id,
            user_agent: fastn_core::vercel::utils::get_user_agent(product.as_str()),
        }
    }

    fn get_endpoint_url(&self, hash: String) -> fastn_core::Result<String> {
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

    pub fn get(
        &self,
        hash: String,
        options: Option<fastn_core::vercel::artifact::ArtifactOptions>,
    ) -> fastn_core::Result<fastn_core::vercel::artifact::ArtifactGetRequest> {
        Ok(fastn_core::vercel::artifact::ArtifactGetRequest(
            fastn_core::vercel::artifact::ArtifactBaseRequest::new(
                self.token.to_string(),
                self.get_endpoint_url(hash)?.clone(),
                self.user_agent.to_string(),
                options,
            ),
        ))
    }

    pub fn put(
        &self,
        hash: String,
        options: Option<fastn_core::vercel::artifact::ArtifactOptions>,
    ) -> fastn_core::Result<fastn_core::vercel::artifact::ArtifactPutRequest> {
        Ok(fastn_core::vercel::artifact::ArtifactPutRequest(
            fastn_core::vercel::artifact::ArtifactBaseRequest::new(
                self.token.to_string(),
                self.get_endpoint_url(hash)?.clone(),
                self.user_agent.to_string(),
                options,
            ),
        ))
    }

    pub fn exists(
        &self,
        hash: String,
        options: Option<fastn_core::vercel::artifact::ArtifactOptions>,
    ) -> fastn_core::Result<fastn_core::vercel::artifact::ArtifactExistsRequest> {
        Ok(fastn_core::vercel::artifact::ArtifactExistsRequest(
            fastn_core::vercel::artifact::ArtifactBaseRequest::new(
                self.token.to_string(),
                self.get_endpoint_url(hash)?.clone(),
                self.user_agent.to_string(),
                options,
            ),
        ))
    }
}
