/// Run database migrations to set up the entity's SQLite schema.
pub fn migrate(db: &rusqlite::Connection) -> rusqlite::Result<()> {
    // https://chatgpt.com/share/687baa1b-ffd8-8000-b57d-546f68170b70
    db.execute_batch(
        r#"
        -- Create tables for entity data storage
        
        -- Example: Entity metadata table
        CREATE TABLE IF NOT EXISTS entity_metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL,
            updated_at INTEGER NOT NULL DEFAULT (unixepoch())
        );
        
        -- Add more tables as needed for entity functionality
        "#,
    )?;

    Ok(())
}
