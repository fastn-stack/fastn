/// Initialize the Automerge document storage tables in SQLite
pub fn initialize_database(conn: &rusqlite::Connection) -> eyre::Result<()> {
    use eyre::WrapErr;

    conn.execute_batch(
        r#"
        -- Automerge documents storage
        CREATE TABLE IF NOT EXISTS fastn_documents (
            path              TEXT PRIMARY KEY,
            automerge_binary  BLOB NOT NULL,
            heads             TEXT NOT NULL,
            actor_id          TEXT NOT NULL,
            updated_at        INTEGER NOT NULL
        );

        -- Sync state for document synchronization (future use)
        CREATE TABLE IF NOT EXISTS fastn_sync_state (
            document_path     TEXT NOT NULL,
            peer_id52         TEXT NOT NULL,
            sync_state        BLOB NOT NULL,
            their_heads       TEXT,
            our_heads         TEXT,
            last_sync_at      INTEGER NOT NULL,
            sync_errors       INTEGER DEFAULT 0,
            
            PRIMARY KEY (document_path, peer_id52)
        );

        CREATE INDEX IF NOT EXISTS idx_last_sync ON fastn_sync_state(last_sync_at);

        -- Cache tables (derived from Automerge for performance)
        
        -- Relationship cache (extracted from /-/relationships/*)
        CREATE TABLE IF NOT EXISTS fastn_relationship_cache (
            their_alias       TEXT PRIMARY KEY,
            my_alias_used     TEXT NOT NULL,
            relationship_type TEXT,
            last_seen         INTEGER,
            extracted_at      INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_my_alias ON fastn_relationship_cache(my_alias_used);

        -- Permission cache (extracted from */meta documents)
        CREATE TABLE IF NOT EXISTS fastn_permission_cache (
            document_path     TEXT NOT NULL,
            grantee_alias     TEXT,
            grantee_group     TEXT,
            permission_level  TEXT NOT NULL,
            extracted_at      INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_perm_path ON fastn_permission_cache(document_path);
        CREATE INDEX IF NOT EXISTS idx_perm_grantee ON fastn_permission_cache(grantee_alias);

        -- Group membership cache (extracted from /-/groups/*)
        CREATE TABLE IF NOT EXISTS fastn_group_cache (
            group_name        TEXT NOT NULL,
            member_alias      TEXT,
            member_group      TEXT,
            extracted_at      INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_group ON fastn_group_cache(group_name);
        CREATE INDEX IF NOT EXISTS idx_member ON fastn_group_cache(member_alias);
        "#,
    )
    .wrap_err("Failed to initialize Automerge database schema")?;

    Ok(())
}
