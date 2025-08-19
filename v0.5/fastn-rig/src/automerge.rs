/// Typed path for rig configuration documents
#[derive(Debug, Clone, PartialEq)]
pub struct RigConfigPath(String);

impl RigConfigPath {
    pub fn new(rig_id52: &fastn_id52::PublicKey) -> Self {
        Self(format!("/-/rig/{}/config", rig_id52.id52()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

/// Typed path for endpoint status documents
#[derive(Debug, Clone, PartialEq)]
pub struct EndpointStatusPath(String);

impl EndpointStatusPath {
    pub fn new(endpoint_id52: &fastn_id52::PublicKey) -> Self {
        Self(format!("/-/endpoints/{}/status", endpoint_id52.id52()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, PartialEq, fastn_automerge::Reconcile, fastn_automerge::Hydrate)]
pub struct RigConfig {
    /// The rig owner's public key
    pub owner: fastn_id52::PublicKey,
    /// Unix timestamp when the rig was created
    pub created_at: i64,
    /// The current active entity (if any)
    pub current_entity: Option<fastn_id52::PublicKey>,
}

impl RigConfig {
    pub fn create(
        db: &fastn_automerge::Db,
        path: RigConfigPath,
        owner: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<()> {
        let config = Self {
            owner: *owner,
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
            current_entity: None,
        };
        db.create(path.as_str(), &config)
    }

    pub fn load(db: &fastn_automerge::Db, path: RigConfigPath) -> fastn_automerge::Result<Self> {
        db.get(path.as_str())
    }

    pub fn update_current_entity(
        db: &fastn_automerge::Db,
        path: RigConfigPath,
        entity: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<()> {
        db.modify::<Self, _>(path.as_str(), |config| {
            config.current_entity = Some(*entity);
        })
    }

    pub fn get_current_entity(
        db: &fastn_automerge::Db,
        path: RigConfigPath,
    ) -> fastn_automerge::Result<Option<fastn_id52::PublicKey>> {
        let config = Self::load(db, path)?;
        Ok(config.current_entity)
    }
}

#[derive(Debug, Clone, PartialEq, fastn_automerge::Reconcile, fastn_automerge::Hydrate)]
pub struct EndpointStatus {
    /// The endpoint's public key
    pub endpoint: fastn_id52::PublicKey,
    /// Whether the endpoint is currently online
    pub is_online: bool,
    /// Unix timestamp when the status was last updated
    pub updated_at: i64,
}

impl EndpointStatus {
    pub fn create(
        db: &fastn_automerge::Db,
        path: EndpointStatusPath,
        endpoint: &fastn_id52::PublicKey,
        is_online: bool,
    ) -> fastn_automerge::Result<()> {
        let status = Self {
            endpoint: *endpoint,
            is_online,
            updated_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64,
        };
        db.create(path.as_str(), &status)
    }

    pub fn load(
        db: &fastn_automerge::Db,
        path: EndpointStatusPath,
    ) -> fastn_automerge::Result<Self> {
        db.get(path.as_str())
    }

    pub fn is_online(
        db: &fastn_automerge::Db,
        path: EndpointStatusPath,
    ) -> fastn_automerge::Result<bool> {
        match Self::load(db, path) {
            Ok(status) => Ok(status.is_online),
            Err(_) => Ok(false), // Default to offline if document doesn't exist
        }
    }

    pub fn set_online(
        db: &fastn_automerge::Db,
        path: EndpointStatusPath,
        endpoint: &fastn_id52::PublicKey,
        online: bool,
    ) -> fastn_automerge::Result<()> {
        // Try to update existing document, create if it doesn't exist
        if db.exists(path.as_str())? {
            db.modify::<Self, _>(path.as_str(), |status| {
                status.is_online = online;
                status.updated_at = std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64;
            })
        } else {
            Self::create(db, path, endpoint, online)
        }
    }
}
