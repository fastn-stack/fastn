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

/// Typed path for entity status documents
#[derive(Debug, Clone, PartialEq)]
pub struct EntityStatusPath(String);

impl EntityStatusPath {
    pub fn new(entity_id52: &fastn_id52::PublicKey) -> Self {
        Self(format!("/-/entities/{}/status", entity_id52.id52()))
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
    /// The current active entity
    pub current_entity: fastn_id52::PublicKey,
}

impl RigConfig {
    pub fn load(
        db: &fastn_automerge::Db,
        rig_id52: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<Self> {
        let path = RigConfigPath::new(rig_id52);
        db.get(path.as_str())
    }

    pub fn save(
        &self,
        db: &fastn_automerge::Db,
        rig_id52: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<()> {
        let path = RigConfigPath::new(rig_id52);
        if db.exists(path.as_str())? {
            db.update(path.as_str(), self)
        } else {
            db.create(path.as_str(), self)
        }
    }

    pub fn update_current_entity(
        db: &fastn_automerge::Db,
        rig_id52: &fastn_id52::PublicKey,
        entity: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<()> {
        let path = RigConfigPath::new(rig_id52);
        db.modify::<Self, _>(path.as_str(), |config| {
            config.current_entity = *entity;
        })
    }

    pub fn get_current_entity(
        db: &fastn_automerge::Db,
        rig_id52: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<fastn_id52::PublicKey> {
        let config = Self::load(db, rig_id52)?;
        Ok(config.current_entity)
    }
}

#[derive(Debug, Clone, PartialEq, fastn_automerge::Reconcile, fastn_automerge::Hydrate)]
pub struct EntityStatus {
    /// The entity's public key
    pub entity: fastn_id52::PublicKey,
    /// Whether the entity is currently online
    pub is_online: bool,
    /// Unix timestamp when the status was last updated
    pub updated_at: i64,
}

impl EntityStatus {
    pub fn load(
        db: &fastn_automerge::Db,
        entity_id52: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<Self> {
        let path = EntityStatusPath::new(entity_id52);
        db.get(path.as_str())
    }

    pub fn save(
        &self,
        db: &fastn_automerge::Db,
        entity_id52: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<()> {
        let path = EntityStatusPath::new(entity_id52);
        if db.exists(path.as_str())? {
            db.update(path.as_str(), self)
        } else {
            db.create(path.as_str(), self)
        }
    }

    pub fn is_online(
        db: &fastn_automerge::Db,
        entity_id52: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<bool> {
        match Self::load(db, entity_id52) {
            Ok(status) => Ok(status.is_online),
            Err(_) => Ok(false), // Default to offline if document doesn't exist
        }
    }

    pub fn set_online(
        db: &fastn_automerge::Db,
        entity_id52: &fastn_id52::PublicKey,
        online: bool,
    ) -> fastn_automerge::Result<()> {
        let path = EntityStatusPath::new(entity_id52);

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
            // Create new status document
            let status = Self {
                entity: *entity_id52,
                is_online: online,
                updated_at: std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            };
            db.create(path.as_str(), &status)
        }
    }
}
