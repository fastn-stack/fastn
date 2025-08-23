#[derive(Debug, thiserror::Error)]
pub enum OpenError {
    #[error("Database not found: {0}. Run 'init' first.")]
    NotFound(std::path::PathBuf),
    #[error("Database at {0} exists but is not initialized. Run 'init' first.")]
    NotInitialized(std::path::PathBuf),
    #[error("Database missing actor counter - not properly initialized")]
    MissingActorCounter,
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),
    #[error("Hydrate error: {0}")]
    Hydrate(#[from] autosurgeon::HydrateError),
    #[error("Invalid entity: {0}")]
    InvalidEntity(String),
}

impl crate::Db {
    /// Open existing database
    pub fn open(db_path: &std::path::Path) -> Result<Self, OpenError> {
        if !db_path.exists() {
            return Err(OpenError::NotFound(db_path.to_path_buf()));
        }

        let conn = rusqlite::Connection::open(db_path).map_err(OpenError::Database)?;

        // Check if database is properly initialized by looking for our tables
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='fastn_documents'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        if !table_exists {
            return Err(OpenError::NotInitialized(db_path.to_path_buf()));
        }

        // Read the actor counter directly from SQL to get stored entity
        let counter_doc_path = crate::DocumentPath::from_string("/-/system/actor_counter")
            .expect("System document path should be valid");

        let binary: Vec<u8> = conn
            .query_row(
                "SELECT automerge_binary FROM fastn_documents WHERE path = ?1",
                [&counter_doc_path],
                |row| row.get(0),
            )
            .map_err(|_| OpenError::MissingActorCounter)?;

        let doc = automerge::AutoCommit::load(&binary).map_err(OpenError::Automerge)?;
        let counter: crate::ActorCounter =
            autosurgeon::hydrate(&doc).map_err(OpenError::Hydrate)?;

        // Parse stored entity ID back to PublicKey
        let entity = std::str::FromStr::from_str(&counter.entity_id52)
            .map_err(|e| OpenError::InvalidEntity(format!("Invalid entity ID52: {e}")))?;

        Ok(Self {
            conn,
            entity,
            device_number: 0, // Primary device
            mutex: std::sync::Mutex::new(()),
        })
    }
}

#[derive(Debug, thiserror::Error)]
pub enum InitError {
    #[error("Database already exists: {0}")]
    DatabaseExists(std::path::PathBuf),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Migration error: {0}")]
    Migration(rusqlite::Error),
    #[error("Create error: {0}")]
    Create(Box<CreateError>),
}

impl crate::Db {
    /// Initialize a new database for an entity (primary device)
    pub fn init(
        db_path: &std::path::Path,
        entity: &fastn_id52::PublicKey,
    ) -> Result<Self, InitError> {
        if db_path.exists() {
            return Err(InitError::DatabaseExists(db_path.to_path_buf()));
        }

        let conn = rusqlite::Connection::open(db_path).map_err(InitError::Database)?;
        crate::migration::initialize_database(&conn).map_err(InitError::Migration)?;

        let db = Self {
            conn,
            entity: *entity,  // Store PublicKey directly
            device_number: 0, // Primary device is always 0
            mutex: std::sync::Mutex::new(()),
        };

        // Initialize the actor counter with database identity
        let counter_doc_path = crate::DocumentPath::from_string("/-/system/actor_counter")
            .expect("System document path should be valid");
        let counter = crate::ActorCounter {
            entity_id52: entity.id52(), // Store as string in the document for now
            next_device: 1,             // Next device will be 1
        };
        db.create_impl(&counter_doc_path, &counter)
            .map_err(|e| InitError::Create(Box::new(e)))?;

        Ok(db)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum CreateError {
    #[error("Document already exists: {0}")]
    DocumentExists(crate::DocumentPath),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),
    #[error("Reconcile error: {0}")]
    Reconcile(#[from] autosurgeon::ReconcileError),
}

impl crate::Db {
    /// Create a new document (internal implementation - use derive macro instead)
    #[doc(hidden)]
    pub fn create_impl<T>(&self, path: &crate::DocumentPath, value: &T) -> Result<(), CreateError>
    where
        T: autosurgeon::Reconcile + serde::Serialize,
    {
        // Ensure actor ID is initialized
        // No need for initialization check - entity is always set during init/open

        // Check if document already exists
        let exists: bool = self
            .conn
            .query_row(
                "SELECT COUNT(*) > 0 FROM fastn_documents WHERE path = ?1",
                [path],
                |row| row.get(0),
            )
            .unwrap_or(false);

        if exists {
            return Err(CreateError::DocumentExists(path.clone()));
        }

        // Create new document with actor
        let mut doc = automerge::AutoCommit::new();
        doc.set_actor(automerge::ActorId::from(self.actor_id().as_bytes()));

        // Reconcile value into document root
        autosurgeon::reconcile(&mut doc, value).map_err(CreateError::Reconcile)?;

        // Get heads as string
        let heads = doc
            .get_heads()
            .into_iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Serialize to JSON for querying (SQLite will store as JSONB)
        let json_data = serde_json::to_string(value).map_err(|e| {
            CreateError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;

        // Save to database
        self.conn.execute(
            "INSERT INTO fastn_documents (path, created_alias, automerge_binary, json_data, heads, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            rusqlite::params![
                path,
                &self.entity.id52(),
                doc.save(),
                json_data,
                heads,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ],
        ).map_err(CreateError::Database)?;

        Ok(())
    }

    /// Create a new document
    #[deprecated(note = "Use the #[derive(Document)] macro and call document.create(&db) instead")]
    pub fn create<T>(&self, path: &crate::DocumentPath, value: &T) -> Result<(), CreateError>
    where
        T: autosurgeon::Reconcile + serde::Serialize,
    {
        self.create_impl(path, value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum GetError {
    #[error("Document not found: {0}")]
    NotFound(crate::DocumentPath),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),
    #[error("Hydrate error: {0}")]
    Hydrate(#[from] autosurgeon::HydrateError),
}

impl crate::Db {
    /// Get a document (internal implementation - use derive macro instead)
    #[doc(hidden)]
    pub fn get_impl<T>(&self, path: &crate::DocumentPath) -> Result<T, GetError>
    where
        T: autosurgeon::Hydrate,
    {
        let binary: Vec<u8> = self
            .conn
            .query_row(
                "SELECT automerge_binary FROM fastn_documents WHERE path = ?1",
                [path],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => GetError::NotFound(path.clone()),
                _ => GetError::Database(e),
            })?;

        let doc = automerge::AutoCommit::load(&binary).map_err(GetError::Automerge)?;
        let value: T = autosurgeon::hydrate(&doc).map_err(GetError::Hydrate)?;
        Ok(value)
    }

    /// Get a document
    #[deprecated(
        note = "Use the #[derive(Document)] macro and call DocumentType::load(&db, &id) instead"
    )]
    pub fn get<T>(&self, path: &crate::DocumentPath) -> Result<T, GetError>
    where
        T: autosurgeon::Hydrate,
    {
        self.get_impl(path)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum UpdateError {
    #[error("Document not found: {0}")]
    NotFound(crate::DocumentPath),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),
    #[error("Reconcile error: {0}")]
    Reconcile(#[from] autosurgeon::ReconcileError),
}

impl crate::Db {
    /// Update a document (internal implementation - use derive macro instead)
    #[doc(hidden)]
    pub fn update_impl<T>(&self, path: &crate::DocumentPath, value: &T) -> Result<(), UpdateError>
    where
        T: autosurgeon::Reconcile + serde::Serialize,
    {
        // Load existing document with creation alias
        let (binary, created_alias): (Vec<u8>, String) = self
            .conn
            .query_row(
                "SELECT automerge_binary, created_alias FROM fastn_documents WHERE path = ?1",
                [path],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => UpdateError::NotFound(path.clone()),
                _ => UpdateError::Database(e),
            })?;

        let mut doc = automerge::AutoCommit::load(&binary).map_err(UpdateError::Automerge)?;

        // Use creation alias for actor to maintain consistency
        let actor_id = format!("{}-{}", created_alias, self.device_number);
        doc.set_actor(automerge::ActorId::from(actor_id.as_bytes()));

        // Clear and reconcile new value
        // Note: This is a full replacement. For partial updates, use modify()
        autosurgeon::reconcile(&mut doc, value).map_err(UpdateError::Reconcile)?;

        // Get heads as string
        let heads = doc
            .get_heads()
            .into_iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Serialize to JSON for querying
        let json_data = serde_json::to_string(value).map_err(|e| {
            UpdateError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;

        // Update in database
        self.conn
            .execute(
                "UPDATE fastn_documents 
             SET automerge_binary = ?1, json_data = ?2, heads = ?3, updated_at = ?4 
             WHERE path = ?5",
                rusqlite::params![
                    doc.save(),
                    json_data,
                    heads,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    path,
                ],
            )
            .map_err(UpdateError::Database)?;

        Ok(())
    }

    /// Update a document
    #[deprecated(note = "Use the #[derive(Document)] macro and call document.update(&db) instead")]
    pub fn update<T>(&self, path: &crate::DocumentPath, value: &T) -> Result<(), UpdateError>
    where
        T: autosurgeon::Reconcile + serde::Serialize,
    {
        self.update_impl(path, value)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ModifyError {
    #[error("Document not found: {0}")]
    NotFound(crate::DocumentPath),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
    #[error("Automerge error: {0}")]
    Automerge(#[from] automerge::AutomergeError),
    #[error("Hydrate error: {0}")]
    Hydrate(#[from] autosurgeon::HydrateError),
    #[error("Reconcile error: {0}")]
    Reconcile(#[from] autosurgeon::ReconcileError),
}

impl crate::Db {
    /// Modify a document with a closure
    #[deprecated(note = "Use the #[derive(Document)] macro and load/modify/save pattern instead")]
    pub fn modify<T, F>(&self, path: &crate::DocumentPath, modifier: F) -> Result<(), ModifyError>
    where
        T: autosurgeon::Hydrate + autosurgeon::Reconcile + serde::Serialize,
        F: FnOnce(&mut T),
    {
        // Load existing
        let (binary, created_alias): (Vec<u8>, String) = self
            .conn
            .query_row(
                "SELECT automerge_binary, created_alias FROM fastn_documents WHERE path = ?1",
                [path],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => ModifyError::NotFound(path.clone()),
                _ => ModifyError::Database(e),
            })?;

        let mut doc = automerge::AutoCommit::load(&binary).map_err(ModifyError::Automerge)?;

        // Use creation alias for actor
        let actor_id = format!("{}-{}", created_alias, self.device_number);
        doc.set_actor(automerge::ActorId::from(actor_id.as_bytes()));

        // Hydrate current value
        let mut value: T = autosurgeon::hydrate(&doc).map_err(ModifyError::Hydrate)?;

        // Apply modifications
        modifier(&mut value);

        // Reconcile back
        autosurgeon::reconcile(&mut doc, &value).map_err(ModifyError::Reconcile)?;

        // Get heads as string
        let heads = doc
            .get_heads()
            .into_iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Serialize to JSON for querying
        let json_data = serde_json::to_string(&value).map_err(|e| {
            ModifyError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;

        // Save back
        self.conn
            .execute(
                "UPDATE fastn_documents 
             SET automerge_binary = ?1, json_data = ?2, heads = ?3, updated_at = ?4 
             WHERE path = ?5",
                rusqlite::params![
                    doc.save(),
                    json_data,
                    heads,
                    std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap()
                        .as_secs() as i64,
                    path,
                ],
            )
            .map_err(ModifyError::Database)?;

        Ok(())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum DeleteError {
    #[error("Document not found: {0}")]
    NotFound(crate::DocumentPath),
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

impl crate::Db {
    /// Delete a document
    pub fn delete(&self, path: &crate::DocumentPath) -> Result<(), DeleteError> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM fastn_documents WHERE path = ?1", [path])
            .map_err(DeleteError::Database)?;

        if rows_affected == 0 {
            Err(DeleteError::NotFound(path.clone()))
        } else {
            Ok(())
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ExistsError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

#[derive(Debug, thiserror::Error)]
pub enum ListError {
    #[error("Database error: {0}")]
    Database(#[from] rusqlite::Error),
}

#[derive(Debug)]
#[allow(dead_code)] // clippy false positive: used for future device management
pub(crate) enum NextActorIdError {
    Get(Box<GetError>),
    Create(Box<CreateError>),
    Update(Box<UpdateError>),
    Exists(Box<ExistsError>),
}

impl crate::Db {
    /// Check if a document exists
    pub fn exists(&self, path: &crate::DocumentPath) -> Result<bool, ExistsError> {
        let count: i32 = self
            .conn
            .query_row(
                "SELECT COUNT(*) FROM fastn_documents WHERE path = ?1",
                [path],
                |row| row.get(0),
            )
            .map_err(ExistsError::Database)?;
        Ok(count > 0)
    }

    /// List documents with optional prefix
    pub fn list(&self, prefix: Option<&str>) -> Result<Vec<String>, ListError> {
        let query = if prefix.is_some() {
            "SELECT path FROM fastn_documents WHERE path LIKE ?1 || '%' ORDER BY path"
        } else {
            "SELECT path FROM fastn_documents ORDER BY path"
        };

        let mut stmt = self.conn.prepare(query).map_err(ListError::Database)?;

        let paths = if let Some(prefix) = prefix {
            stmt.query_map([prefix], |row| row.get(0))
                .map_err(ListError::Database)?
                .collect::<std::result::Result<Vec<String>, _>>()
        } else {
            stmt.query_map([], |row| row.get(0))
                .map_err(ListError::Database)?
                .collect::<std::result::Result<Vec<String>, _>>()
        }
        .map_err(ListError::Database)?;

        Ok(paths)
    }

    /// List documents matching a SQL LIKE pattern
    pub fn list_with_pattern(&self, pattern: &str) -> Result<Vec<String>, ListError> {
        let query = "SELECT path FROM fastn_documents WHERE path LIKE ?1 ORDER BY path";
        let mut stmt = self.conn.prepare(query).map_err(ListError::Database)?;
        
        let paths = stmt.query_map([pattern], |row| row.get(0))
            .map_err(ListError::Database)?
            .collect::<std::result::Result<Vec<String>, _>>()
            .map_err(ListError::Database)?;

        Ok(paths)
    }

    /// Find documents where a field equals a specific value
    pub fn find_where<V>(&self, field_path: &str, value: V) -> Result<Vec<crate::DocumentPath>, ListError> 
    where
        V: serde::Serialize,
    {
        let json_path = if field_path.starts_with('$') {
            field_path.to_string()
        } else {
            format!("$.{field_path}")
        };
        
        // Convert value to what json_extract returns (the raw JSON value, not JSON-encoded)
        let value_for_comparison = match serde_json::to_value(value).map_err(|e| {
            ListError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })? {
            serde_json::Value::String(s) => s,
            serde_json::Value::Number(n) => n.to_string(),
            serde_json::Value::Bool(b) => if b { "true".to_string() } else { "false".to_string() },
            serde_json::Value::Null => "null".to_string(),
            v => serde_json::to_string(&v).map_err(|e| {
                ListError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
            })?,
        };
        
        let query = "SELECT path FROM fastn_documents WHERE json_extract(json_data, ?) = ? ORDER BY path";
        let mut stmt = self.conn.prepare(query).map_err(ListError::Database)?;
        
        let paths: Result<Vec<_>, _> = stmt.query_map([&json_path, &value_for_comparison], |row| {
            let path_str: String = row.get(0)?;
            crate::DocumentPath::from_string(&path_str).map_err(|_| {
                rusqlite::Error::InvalidPath("Invalid document path in database".into())
            })
        })
        .map_err(ListError::Database)?
        .collect();

        paths.map_err(ListError::Database)
    }

    /// Find documents where a field exists (is not null)
    pub fn find_exists(&self, field_path: &str) -> Result<Vec<crate::DocumentPath>, ListError> {
        let json_path = if field_path.starts_with('$') {
            field_path.to_string()
        } else {
            format!("$.{field_path}")
        };
        
        let query = "SELECT path FROM fastn_documents WHERE json_extract(json_data, ?) IS NOT NULL ORDER BY path";
        let mut stmt = self.conn.prepare(query).map_err(ListError::Database)?;
        
        let paths: Result<Vec<_>, _> = stmt.query_map([&json_path], |row| {
            let path_str: String = row.get(0)?;
            crate::DocumentPath::from_string(&path_str).map_err(|_| {
                rusqlite::Error::InvalidPath("Invalid document path in database".into())
            })
        })
        .map_err(ListError::Database)?
        .collect();

        paths.map_err(ListError::Database)
    }

    /// Find documents where an array field contains a specific value
    pub fn find_contains<V>(&self, field_path: &str, value: V) -> Result<Vec<crate::DocumentPath>, ListError>
    where
        V: serde::Serialize,
    {
        let json_path = if field_path.starts_with('$') {
            field_path.to_string()
        } else {
            format!("$.{field_path}")
        };
        
        let value_json = serde_json::to_value(value).map_err(|e| {
            ListError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;
        
        let value_str = serde_json::to_string(&value_json).map_err(|e| {
            ListError::Database(rusqlite::Error::ToSqlConversionFailure(Box::new(e)))
        })?;
        
        let query = r#"
            SELECT path FROM fastn_documents 
            WHERE EXISTS (
                SELECT 1 FROM json_each(json_extract(json_data, ?)) 
                WHERE value = json(?)
            ) 
            ORDER BY path
        "#;
        
        let mut stmt = self.conn.prepare(query).map_err(ListError::Database)?;
        
        let paths: Result<Vec<_>, _> = stmt.query_map([&json_path, &value_str], |row| {
            let path_str: String = row.get(0)?;
            crate::DocumentPath::from_string(&path_str).map_err(|_| {
                rusqlite::Error::InvalidPath("Invalid document path in database".into())
            })
        })
        .map_err(ListError::Database)?
        .collect();

        paths.map_err(ListError::Database)
    }

    /// Get raw AutoCommit document for advanced operations
    #[allow(dead_code)] // clippy false positive: used for advanced document operations
    pub(crate) fn get_document(
        &self,
        path: &crate::DocumentPath,
    ) -> Result<automerge::AutoCommit, GetError> {
        let binary: Vec<u8> = self
            .conn
            .query_row(
                "SELECT automerge_binary FROM fastn_documents WHERE path = ?1",
                [path],
                |row| row.get(0),
            )
            .map_err(|e| match e {
                rusqlite::Error::QueryReturnedNoRows => GetError::NotFound(path.clone()),
                _ => GetError::Database(e),
            })?;

        automerge::AutoCommit::load(&binary).map_err(GetError::Automerge)
    }

    /// Get document history with detailed operations
    ///
    /// If `up_to_head` is provided, shows history up to that specific head/change.
    /// If None, shows complete history up to current heads.
    pub fn history(
        &self,
        path: &crate::DocumentPath,
        up_to_head: Option<&str>,
    ) -> Result<crate::DocumentHistory, GetError> {
        let (binary, created_alias, updated_at): (Vec<u8>, String, i64) = self.conn.query_row(
            "SELECT automerge_binary, created_alias, updated_at FROM fastn_documents WHERE path = ?1",
            [path],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).map_err(|e| match e {
            rusqlite::Error::QueryReturnedNoRows => GetError::NotFound(path.clone()),
            _ => GetError::Database(e),
        })?;

        let mut doc = automerge::AutoCommit::load(&binary).map_err(GetError::Automerge)?;

        // Get the heads to show history up to
        let target_heads = doc.get_heads();

        // Get all changes up to the target heads
        let changes = doc.get_changes(&[]);

        let mut edits = Vec::new();
        for (i, change) in changes.iter().enumerate() {
            // Check if this change is an ancestor of our target heads
            let change_hash = change.hash();

            // For now, include all changes if no specific head, or check if it matches
            if up_to_head.is_none() || up_to_head == Some(&change_hash.to_string()) {
                let operations = extract_operations_from_change(change)?;

                edits.push(crate::Edit {
                    index: i + 1,
                    hash: change_hash.to_string(),
                    actor_id: change.actor_id().to_string(),
                    timestamp: 0, // TODO: automerge 0.6.1 doesn't expose timestamp
                    message: change.message().map(String::from),
                    operations,
                });

                // If we found the specific head we're looking for, stop here
                if up_to_head == Some(&change_hash.to_string()) {
                    break;
                }
            }
        }

        Ok(crate::DocumentHistory {
            path: path.to_string(),
            created_alias,
            updated_at,
            heads: target_heads.iter().map(|h| h.to_string()).collect(),
            edits,
        })
    }
}

/// Extract human-readable operations from an Automerge change
fn extract_operations_from_change(
    change: &automerge::Change,
) -> Result<Vec<crate::Operation>, GetError> {
    let mut operations = Vec::new();

    // Note: Automerge 0.6.1 doesn't expose detailed operation information easily
    // We'll need to parse the raw operations from the change
    // For now, return a placeholder showing the operation count

    // In a real implementation, we would iterate through the operations
    // and convert them to our Operation enum. This requires accessing
    // the internal structure of the Change object.

    // Placeholder: Just indicate how many operations occurred
    let op_count = change.len();
    if op_count > 0 {
        operations.push(crate::Operation::Set {
            path: vec![],
            key: format!("({op_count} operations in this change)"),
            value: "Details not yet implemented".to_string(),
        });
    }

    Ok(operations)
}

impl crate::Db {
    /// Get the next actor ID for this database's entity and increment the counter (thread-safe)
    #[allow(dead_code)] // clippy false positive: used for device ID management
    pub(crate) fn next_actor_id(&self, entity_id52: &str) -> Result<String, NextActorIdError> {
        // Lock for atomic operation
        let _lock = self.mutex.lock().unwrap();

        let counter_doc_id = crate::DocumentPath::from_string("/-/system/actor_counter")
            .expect("System document ID should be valid");

        // Load or create actor counter document
        let mut counter = match self.get_impl::<crate::ActorCounter>(&counter_doc_id) {
            Ok(counter) => counter,
            Err(_) => {
                // Create new counter starting at 0
                crate::ActorCounter {
                    entity_id52: entity_id52.to_string(),
                    next_device: 0,
                }
            }
        };

        // Get current device number
        let current_device = counter.next_device;

        // Increment for next time
        counter.next_device += 1;

        // Save the updated counter
        if self
            .exists(&counter_doc_id)
            .map_err(|e| NextActorIdError::Exists(Box::new(e)))?
        {
            self.update_impl(&counter_doc_id, &counter)
                .map_err(|e| NextActorIdError::Update(Box::new(e)))?;
        } else {
            self.create_impl(&counter_doc_id, &counter)
                .map_err(|e| NextActorIdError::Create(Box::new(e)))?;
        }

        // Return the actor ID for the current device
        Ok(format!("{entity_id52}-{current_device}"))
    }
}

// SaveError for the derive macro's save() method
#[derive(Debug, thiserror::Error)]
pub enum SaveError {
    #[error("Exists check failed: {0}")]
    Exists(ExistsError),
    #[error("Create failed: {0}")]
    Create(CreateError),
    #[error("Update failed: {0}")]
    Update(UpdateError),
}
