/// Run migrations for the Rig database
pub fn migrate_database(conn: &rusqlite::Connection) -> eyre::Result<()> {
    use eyre::WrapErr;

    conn.execute_batch(
        r#"
        -- Endpoint online/offline status and current entity tracking
        CREATE TABLE IF NOT EXISTS fastn_endpoints (
            id52              TEXT PRIMARY KEY,
            is_online         INTEGER NOT NULL DEFAULT 0,
            is_current        INTEGER NOT NULL DEFAULT 0
        );

        CREATE INDEX IF NOT EXISTS idx_endpoints_online ON fastn_endpoints(is_online);
        
        -- Ensure only one endpoint can be current at a time
        CREATE UNIQUE INDEX IF NOT EXISTS idx_endpoints_current_unique 
            ON fastn_endpoints(is_current) 
            WHERE is_current = 1;
        "#,
    )
    .wrap_err("Failed to initialize rig database schema")?;

    Ok(())
}
