/// Document ID constructor for default mail configuration
pub fn default_mail_id() -> fastn_automerge::DocumentId {
    fastn_automerge::DocumentId::from_string("/-/mails/default")
        .expect("Default mail document ID should be valid")
}

/// Document ID constructor for alias readme documents
pub fn alias_readme_id(alias_id52: &fastn_id52::PublicKey) -> fastn_automerge::DocumentId {
    let id_str = format!("/-/aliases/{}/readme", alias_id52.id52());
    fastn_automerge::DocumentId::from_string(&id_str)
        .expect("Generated alias readme document ID should be valid")
}

/// Document ID constructor for alias notes documents
pub fn alias_notes_id(alias_id52: &fastn_id52::PublicKey) -> fastn_automerge::DocumentId {
    let id_str = format!("/-/aliases/{}/notes", alias_id52.id52());
    fastn_automerge::DocumentId::from_string(&id_str)
        .expect("Generated alias notes document ID should be valid")
}

#[derive(Debug, Clone, PartialEq, fastn_automerge::Reconcile, fastn_automerge::Hydrate)]
pub struct DefaultMail {
    /// Hashed password for authentication
    pub password_hash: String,
    /// Whether the mail service is active
    pub is_active: bool,
    /// Unix timestamp when created
    pub created_at: i64,
}

impl DefaultMail {
    pub fn load(db: &fastn_automerge::Db) -> fastn_automerge::Result<Self> {
        let doc_id = default_mail_id();
        db.get(&doc_id)
    }

    pub fn save(&self, db: &fastn_automerge::Db) -> fastn_automerge::Result<()> {
        let doc_id = default_mail_id();
        if db.exists(&doc_id)? {
            db.update(&doc_id, self)
        } else {
            db.create(&doc_id, self)
        }
    }
}

#[derive(Debug, Clone, PartialEq, fastn_automerge::Reconcile, fastn_automerge::Hydrate)]
pub struct AliasReadme {
    /// The alias public key
    pub alias: fastn_id52::PublicKey,
    /// Display name for this alias (optional)
    pub name: Option<String>,
    /// Bio or description (optional)
    pub bio: Option<String>,
}

impl AliasReadme {
    pub fn load(
        db: &fastn_automerge::Db,
        alias_id52: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<Self> {
        let doc_id = alias_readme_id(alias_id52);
        db.get(&doc_id)
    }

    pub fn save(&self, db: &fastn_automerge::Db) -> fastn_automerge::Result<()> {
        let doc_id = alias_readme_id(&self.alias);
        if db.exists(&doc_id)? {
            db.update(&doc_id, self)
        } else {
            db.create(&doc_id, self)
        }
    }
}

#[derive(Debug, Clone, PartialEq, fastn_automerge::Reconcile, fastn_automerge::Hydrate)]
pub struct AliasNotes {
    /// The alias public key
    pub alias: fastn_id52::PublicKey,
    /// Nickname or short name for this alias (optional)
    pub nickname: Option<String>,
    /// Private notes about this alias (optional)
    pub notes: Option<String>,
    /// Unix timestamp when this alias became part of our relationships
    pub relationship_started_at: i64,
}

impl AliasNotes {
    pub fn load(
        db: &fastn_automerge::Db,
        alias_id52: &fastn_id52::PublicKey,
    ) -> fastn_automerge::Result<Self> {
        let doc_id = alias_notes_id(alias_id52);
        db.get(&doc_id)
    }

    pub fn save(&self, db: &fastn_automerge::Db) -> fastn_automerge::Result<()> {
        let doc_id = alias_notes_id(&self.alias);
        if db.exists(&doc_id)? {
            db.update(&doc_id, self)
        } else {
            db.create(&doc_id, self)
        }
    }
}
