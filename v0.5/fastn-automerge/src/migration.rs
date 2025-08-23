/// Initialize the Automerge document storage tables in SQLite
pub fn initialize_database(conn: &rusqlite::Connection) -> Result<(), rusqlite::Error> {
    conn.execute_batch(
        r#"
        -- Automerge documents storage
        CREATE TABLE IF NOT EXISTS fastn_documents (
            path              TEXT PRIMARY KEY,
            created_alias     TEXT NOT NULL,      -- Alias used at creation (for actor ID)
            automerge_binary  BLOB NOT NULL,
            json_data         TEXT NOT NULL,      -- JSON representation for querying
            heads             TEXT NOT NULL,
            updated_at        INTEGER NOT NULL
        );

        CREATE INDEX IF NOT EXISTS idx_documents_updated ON fastn_documents(updated_at);
        
        -- Sync state for document synchronization (future use)
        CREATE TABLE IF NOT EXISTS fastn_sync_state (
            document_path     TEXT NOT NULL,
            peer_alias        TEXT NOT NULL,
            our_alias_used    TEXT NOT NULL,
            their_heads       TEXT,
            our_heads         TEXT,
            last_sync_at      INTEGER NOT NULL,
            needs_sync        INTEGER DEFAULT 1,
            
            PRIMARY KEY (document_path, peer_alias)
        );

        CREATE INDEX IF NOT EXISTS idx_sync_needed ON fastn_sync_state(needs_sync, last_sync_at);
        
        -- Document access tracking
        CREATE TABLE IF NOT EXISTS fastn_document_access (
            document_path     TEXT NOT NULL,
            peer_alias        TEXT NOT NULL,
            our_alias_used    TEXT NOT NULL,
            permission        TEXT NOT NULL,      -- 'read', 'write', 'admin'
            granted_at        INTEGER NOT NULL,
            last_shared_at    INTEGER,
            
            PRIMARY KEY (document_path, peer_alias)
        );
        
        -- Cache tables (derived from Automerge for performance)
        
        -- Alias cache (extracted from /-/{alias-id52}/notes)
        CREATE TABLE IF NOT EXISTS fastn_alias_cache (
            alias_id52        TEXT PRIMARY KEY,
            relationship      TEXT,
            can_manage_groups INTEGER DEFAULT 0,
            can_grant_access  INTEGER DEFAULT 0,
            is_admin          INTEGER DEFAULT 0,
            trusted           INTEGER DEFAULT 0,
            last_interaction  INTEGER,
            extracted_at      INTEGER NOT NULL
        );
        
        CREATE INDEX IF NOT EXISTS idx_trusted ON fastn_alias_cache(trusted);
        
        -- Permission cache (extracted from {doc}/-/meta documents)
        CREATE TABLE IF NOT EXISTS fastn_permission_cache (
            document_path     TEXT NOT NULL,
            grantee_alias     TEXT,
            grantee_group     TEXT,
            permission_level  TEXT NOT NULL,
            granted_by        TEXT NOT NULL,
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
        
        CREATE UNIQUE INDEX IF NOT EXISTS idx_group_member ON fastn_group_cache(group_name, member_alias, member_group);
        
        CREATE INDEX IF NOT EXISTS idx_group ON fastn_group_cache(group_name);
        CREATE INDEX IF NOT EXISTS idx_member ON fastn_group_cache(member_alias);
        "#,
    )?;

    Ok(())
}
