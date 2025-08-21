/// Document ID constructor for rig configuration documents
pub fn rig_config_id(rig_id52: &fastn_id52::PublicKey) -> fastn_automerge::DocumentId {
    let id_str = format!("/-/rig/{}/config", rig_id52.id52());
    fastn_automerge::DocumentId::from_string(&id_str)
        .expect("Generated rig config document ID should be valid")
}

/// Document ID constructor for entity status documents
pub fn entity_status_id(entity_id52: &fastn_id52::PublicKey) -> fastn_automerge::DocumentId {
    let id_str = format!("/-/entities/{}/status", entity_id52.id52());
    fastn_automerge::DocumentId::from_string(&id_str)
        .expect("Generated entity status document ID should be valid")
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
        let doc_id = rig_config_id(rig_id52);
        db.get(&doc_id)
    }

    pub fn save(&self, db: &fastn_automerge::Db) -> fastn_automerge::Result<()> {
        let doc_id = rig_config_id(&self.owner);
        if db.exists(&doc_id)? {
            db.update(&doc_id, self)
        } else {
            db.create(&doc_id, self)
        }
    }

    pub fn update_current_entity(
        db: &fastn_automerge::Db,
        rig_id52: &fastn_id52::PublicKey,
        entity: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<()> {
        let doc_id = rig_config_id(rig_id52);
        db.modify::<Self, _>(&doc_id, |config| {
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
        let doc_id = entity_status_id(entity_id52);
        db.get(&doc_id)
    }

    pub fn save(&self, db: &fastn_automerge::Db) -> fastn_automerge::Result<()> {
        let doc_id = entity_status_id(&self.entity);
        if db.exists(&doc_id)? {
            db.update(&doc_id, self)
        } else {
            db.create(&doc_id, self)
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
        let doc_id = entity_status_id(entity_id52);

        // Try to update existing document, create if it doesn't exist
        if db.exists(&doc_id)? {
            db.modify::<Self, _>(&doc_id, |status| {
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
            db.create(&doc_id, &status)
        }
    }
}
