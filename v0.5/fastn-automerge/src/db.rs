#[derive(Debug)]
pub enum LoadError {
    NotFound(std::path::PathBuf),
    NotInitialized(std::path::PathBuf),
    MissingActorCounter,
    DatabaseError(rusqlite::Error),
}

impl crate::Db {
    /// Open existing database
    pub fn open(db_path: &std::path::Path) -> crate::Result<Self> {
        if !db_path.exists() {
            return Err(eyre::eyre!("Database not found: {}. Run 'init' first.", db_path.display()));
        }

        let conn = rusqlite::Connection::open(db_path)?;

        // Check if database is properly initialized by looking for our tables
        let table_exists: bool = conn.query_row(
            "SELECT COUNT(*) > 0 FROM sqlite_master WHERE type='table' AND name='fastn_documents'",
            [],
            |row| row.get(0),
        ).unwrap_or(false);

        if !table_exists {
            return Err(eyre::eyre!("Database at {} exists but is not initialized. Run 'init' first.", db_path.display()));
        }

        // Read the actor counter directly from SQL to get stored entity
        let counter_doc_path = crate::DocumentPath::from_string("/-/system/actor_counter")
            .expect("System document path should be valid");
            
        let binary: Vec<u8> = conn.query_row(
            "SELECT automerge_binary FROM fastn_documents WHERE path = ?1",
            [&counter_doc_path],
            |row| row.get(0),
        ).map_err(|e| eyre::eyre!("Missing actor counter: {e}"))?;

        let doc = automerge::AutoCommit::load(&binary)?;
        let counter: crate::ActorCounter = autosurgeon::hydrate(&doc)?;

        // Parse stored entity ID back to PublicKey
        let entity = std::str::FromStr::from_str(&counter.entity_id52)?;
            
        Ok(Self {
            conn,
            entity,
            device_number: 0, // Primary device
            mutex: std::sync::Mutex::new(()),
        })
    }

}

#[derive(Debug)]
pub enum InitError {
    DatabaseExists(std::path::PathBuf),
    Database(rusqlite::Error),
    Migration(eyre::Report),
    Create(CreateError),
}

impl crate::Db {
    /// Initialize a new database for an entity (primary device)
    pub fn init(db_path: &std::path::Path, entity: &fastn_id52::PublicKey) -> crate::Result<Self> {
        if db_path.exists() {
            return Err(eyre::eyre!("Database already exists at {}", db_path.display()));
        }

        let conn = rusqlite::Connection::open(db_path)?;
        crate::migration::initialize_database(&conn)?;
        
        let db = Self { 
            conn, 
            entity: *entity, // Store PublicKey directly
            device_number: 0, // Primary device is always 0
            mutex: std::sync::Mutex::new(()),
        };
        
        // Initialize the actor counter with database identity
        let counter_doc_path = crate::DocumentPath::from_string("/-/system/actor_counter")
            .expect("System document path should be valid");
        let counter = crate::ActorCounter { 
            entity_id52: entity.id52(), // Store as string in the document for now
            next_device: 1, // Next device will be 1
        };
        db.create(&counter_doc_path, &counter)?;
        
        Ok(db)
    }
}

#[derive(Debug)]
pub enum CreateError {
    DocumentExists(crate::DocumentPath),
    Database(rusqlite::Error),
    Automerge(automerge::AutomergeError),
    Reconcile(autosurgeon::ReconcileError),
}

impl crate::Db {
    /// Create a new document
    pub fn create<T>(&self, path: &crate::DocumentPath, value: &T) -> crate::Result<()>
    where
        T: autosurgeon::Reconcile,
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
            return Err(eyre::eyre!("Document already exists: {}", path));
        }

        // Create new document with actor
        let mut doc = automerge::AutoCommit::new();
        doc.set_actor(automerge::ActorId::from(self.actor_id().as_bytes()));

        // Reconcile value into document root
        autosurgeon::reconcile(&mut doc, value)?;

        // Get heads as string
        let heads = doc
            .get_heads()
            .into_iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Save to database
        self.conn.execute(
            "INSERT INTO fastn_documents (path, created_alias, automerge_binary, heads, updated_at) 
             VALUES (?1, ?2, ?3, ?4, ?5)",
            rusqlite::params![
                path,
                &self.entity.id52(),
                doc.save(),
                heads,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
            ],
        )?;

        Ok(())
    }

}

#[derive(Debug)]
pub enum GetError {
    NotFound(crate::DocumentPath),
    Database(rusqlite::Error),
    Automerge(automerge::AutomergeError),
    Hydrate(autosurgeon::HydrateError),
}

impl crate::Db {
    /// Get a document
    pub fn get<T>(&self, path: &crate::DocumentPath) -> crate::Result<T>
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
?;

        let doc = automerge::AutoCommit::load(&binary)?;
        let value: T = autosurgeon::hydrate(&doc)?;
        Ok(value)
    }

}

#[derive(Debug)]
pub enum UpdateError {
    NotFound(crate::DocumentPath),
    Database(rusqlite::Error),
    Automerge(automerge::AutomergeError),
    Reconcile(autosurgeon::ReconcileError),
}

impl crate::Db {
    /// Update a document
    pub fn update<T>(&self, path: &crate::DocumentPath, value: &T) -> crate::Result<()>
    where
        T: autosurgeon::Reconcile,
    {
        // Load existing document with creation alias
        let (binary, created_alias): (Vec<u8>, String) = self
            .conn
            .query_row(
                "SELECT automerge_binary, created_alias FROM fastn_documents WHERE path = ?1",
                [path],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            ?;

        let mut doc = automerge::AutoCommit::load(&binary)?;

        // Use creation alias for actor to maintain consistency
        let actor_id = format!(
            "{}-{}",
            created_alias,
            self.device_number
        );
        doc.set_actor(automerge::ActorId::from(actor_id.as_bytes()));

        // Clear and reconcile new value
        // Note: This is a full replacement. For partial updates, use modify()
        autosurgeon::reconcile(&mut doc, value)?;

        // Get heads as string
        let heads = doc
            .get_heads()
            .into_iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Update in database
        self.conn.execute(
            "UPDATE fastn_documents 
             SET automerge_binary = ?1, heads = ?2, updated_at = ?3 
             WHERE path = ?4",
            rusqlite::params![
                doc.save(),
                heads,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                path,
            ],
        )?;

        Ok(())
    }

    /// Modify a document with a closure
    pub fn modify<T, F>(&self, path: &crate::DocumentPath, modifier: F) -> crate::Result<()>
    where
        T: autosurgeon::Hydrate + autosurgeon::Reconcile,
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
            ?;

        let mut doc = automerge::AutoCommit::load(&binary)?;

        // Use creation alias for actor
        let actor_id = format!(
            "{}-{}",
            created_alias,
            self.device_number
        );
        doc.set_actor(automerge::ActorId::from(actor_id.as_bytes()));

        // Hydrate current value
        let mut value: T = autosurgeon::hydrate(&doc)?;

        // Apply modifications
        modifier(&mut value);

        // Reconcile back
        autosurgeon::reconcile(&mut doc, &value)?;

        // Get heads as string
        let heads = doc
            .get_heads()
            .into_iter()
            .map(|h| h.to_string())
            .collect::<Vec<_>>()
            .join(",");

        // Save back
        self.conn.execute(
            "UPDATE fastn_documents 
             SET automerge_binary = ?1, heads = ?2, updated_at = ?3 
             WHERE path = ?4",
            rusqlite::params![
                doc.save(),
                heads,
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs() as i64,
                path,
            ],
        )?;

        Ok(())
    }

}

#[derive(Debug)]
pub enum DeleteError {
    NotFound(crate::DocumentPath),
    Database(rusqlite::Error),
}

impl crate::Db {
    /// Delete a document
    pub fn delete(&self, path: &crate::DocumentPath) -> crate::Result<()> {
        let rows_affected = self
            .conn
            .execute("DELETE FROM fastn_documents WHERE path = ?1", [path])
            ?;

        if rows_affected == 0 {
            Err(eyre::eyre!("Document not found: {}", path))
        } else {
            Ok(())
        }
    }

}

#[derive(Debug)]
pub enum ExistsError {
    Database(rusqlite::Error),
}

impl crate::Db {
    /// Check if a document exists
    pub fn exists(&self, path: &crate::DocumentPath) -> crate::Result<bool> {
        let count: i32 = self.conn.query_row(
            "SELECT COUNT(*) FROM fastn_documents WHERE path = ?1",
            [path],
            |row| row.get(0),
)?;
        Ok(count > 0)
    }

    /// List documents with optional prefix
    pub fn list(&self, prefix: Option<&str>) -> crate::Result<Vec<String>> {
        let query = if prefix.is_some() {
            "SELECT path FROM fastn_documents WHERE path LIKE ?1 || '%' ORDER BY path"
        } else {
            "SELECT path FROM fastn_documents ORDER BY path"
        };

        let mut stmt = self.conn.prepare(query)?;

        let paths = if let Some(prefix) = prefix {
            stmt.query_map([prefix], |row| row.get(0))?
                .collect::<std::result::Result<Vec<String>, _>>()
        } else {
            stmt.query_map([], |row| row.get(0))?
                .collect::<std::result::Result<Vec<String>, _>>()
        }?;

        Ok(paths)
    }

    /// Get raw AutoCommit document for advanced operations
    pub fn get_document(&self, path: &crate::DocumentPath) -> crate::Result<automerge::AutoCommit> {
        let binary: Vec<u8> = self
            .conn
            .query_row(
                "SELECT automerge_binary FROM fastn_documents WHERE path = ?1",
                [path],
                |row| row.get(0),
            )
            ?;

        Ok(automerge::AutoCommit::load(&binary)?)
    }

    /// Clear all documents from the database
    pub fn clear(&self) -> crate::Result<usize> {
        let count = self
            .conn
            .query_row("SELECT COUNT(*) FROM fastn_documents", [], |row| {
                row.get::<_, usize>(0)
            })?;

        self.conn.execute("DELETE FROM fastn_documents", [])?;
        self.conn.execute("DELETE FROM fastn_sync_state", [])?;
        self.conn.execute("DELETE FROM fastn_document_access", [])?;
        self.conn.execute("DELETE FROM fastn_alias_cache", [])?;
        self.conn
            .execute("DELETE FROM fastn_permission_cache", [])?;
        self.conn.execute("DELETE FROM fastn_group_cache", [])?;

        Ok(count)
    }

    /// Get document history with detailed operations
    ///
    /// If `up_to_head` is provided, shows history up to that specific head/change.
    /// If None, shows complete history up to current heads.
    pub fn history(
        &self,
        path: &crate::DocumentPath,
        up_to_head: Option<&str>,
    ) -> crate::Result<crate::DocumentHistory> {
        let (binary, created_alias, updated_at): (Vec<u8>, String, i64) = self.conn.query_row(
            "SELECT automerge_binary, created_alias, updated_at FROM fastn_documents WHERE path = ?1",
            [path],
            |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
        ).map_err(|_| Box::new(crate::Error::NotFound(path.to_string())))?;

        let mut doc = automerge::AutoCommit::load(&binary)?;

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
) -> crate::Result<Vec<crate::Operation>> {
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
    pub fn next_actor_id(&self, entity_id52: &str) -> crate::Result<String> {
        // Lock for atomic operation
        let _lock = self.mutex.lock().unwrap();
        
        let counter_doc_id = crate::DocumentPath::from_string("/-/system/actor_counter")
            .expect("System document ID should be valid");
        
        // Load or create actor counter document
        let mut counter = match self.get::<crate::ActorCounter>(&counter_doc_id) {
            Ok(counter) => counter,
            Err(_) => {
                // Create new counter starting at 0
                crate::ActorCounter { 
                    entity_id52: entity_id52.to_string(),
                    next_device: 0 
                }
            }
        };
        
        // Get current device number
        let current_device = counter.next_device;
        
        // Increment for next time
        counter.next_device += 1;
        
        // Save the updated counter
        if self.exists(&counter_doc_id)? {
            self.update(&counter_doc_id, &counter)?;
        } else {
            self.create(&counter_doc_id, &counter)?;
        }
        
        // Return the actor ID for the current device
        Ok(format!("{}-{}", entity_id52, current_device))
    }
}
