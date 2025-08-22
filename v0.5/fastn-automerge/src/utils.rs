impl std::fmt::Display for crate::Operation {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            crate::Operation::Set { path, key, value } => {
                let full_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path.join("."), key)
                };
                write!(f, "Set {full_path} = {value}")
            }
            crate::Operation::Delete { path, key } => {
                let full_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path.join("."), key)
                };
                write!(f, "Delete {full_path}")
            }
            crate::Operation::Insert { path, index, value } => {
                let path_str = if path.is_empty() {
                    String::from("[]")
                } else {
                    path.join(".")
                };
                write!(f, "Insert {path_str}[{index}] = {value}")
            }
            crate::Operation::Remove { path, index } => {
                let path_str = if path.is_empty() {
                    String::from("[]")
                } else {
                    path.join(".")
                };
                write!(f, "Remove {path_str}[{index}]")
            }
            crate::Operation::Increment { path, key, delta } => {
                let full_path = if path.is_empty() {
                    key.clone()
                } else {
                    format!("{}.{}", path.join("."), key)
                };
                write!(f, "Increment {full_path} by {delta}")
            }
        }
    }
}

impl std::fmt::Display for crate::Edit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "[{}] {} by {}",
            self.index,
            &self.hash[..8.min(self.hash.len())],
            self.actor_id
        )?;
        writeln!(f, "  Time: {}", self.timestamp)?;
        if let Some(msg) = &self.message {
            writeln!(f, "  Message: {msg}")?;
        }
        writeln!(f, "  Operations:")?;
        for op in &self.operations {
            writeln!(f, "    - {op}")?;
        }
        Ok(())
    }
}

impl std::fmt::Display for crate::DocumentHistory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(f, "Document: {}", self.path)?;
        writeln!(f, "Created by: {}", self.created_alias)?;
        writeln!(f, "Last updated: {}", self.updated_at)?;
        writeln!(f, "Heads: {} head(s)", self.heads.len())?;
        for head in &self.heads {
            writeln!(f, "  - {}", &head[..8.min(head.len())])?;
        }
        writeln!(f, "\n=== History ({} edits) ===", self.edits.len())?;
        for edit in &self.edits {
            writeln!(f, "\n{edit}")?;
        }
        Ok(())
    }
}


/// Create a test database with a random entity (for testing only)
pub fn create_test_db() -> crate::Result<(crate::Db, tempfile::TempDir)> {
    let temp_dir = tempfile::TempDir::new()
        .map_err(|e| eyre::eyre!("Failed to create temp directory: {e}"))?;
    let db_path = temp_dir.path().join("test.db");
    
    // Create a test entity
    let test_entity = fastn_id52::SecretKey::generate().public_key();
    let db = crate::Db::init(&db_path, &test_entity)?;
    
    Ok((db, temp_dir))
}
