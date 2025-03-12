pub(crate) fn fastn_migrations() -> Vec<fastn_core::package::MigrationData> {
    vec![fastn_core::package::MigrationData {
        number: 0,
        name: "initial".to_string(),
        content: r#"
            CREATE TABLE IF NOT EXISTS fastn_email_queue
            (
                id           INTEGER           PRIMARY KEY,
                from_address TEXT              NOT NULL,
                from_name    TEXT              NOT NULL,
                reply_to     TEXT,
                to_address   TEXT              NOT NULL,
                cc_address   TEXT,
                bcc_address  TEXT,
                subject      TEXT              NOT NULL,
                body_text    TEXT              NOT NULL,
                body_html    TEXT              NOT NULL,
                retry_count  INTEGER DEFAULT 0 NOT NULL,
                created_at   INTEGER           NOT NULL,
                updated_at   INTEGER           NOT NULL,
                sent_at      INTEGER           NOT NULL,
                mkind        TEXT              NOT NULL,
                status       TEXT              NOT NULL
            ) STRICT;

            "#
        .to_string(),
    }]
}

pub const MIGRATION_TABLE: &str = r#"

CREATE TABLE IF NOT EXISTS fastn_migration
(
    id               INTEGER PRIMARY KEY,
    app_name         TEXT NOT NULL,
    migration_number INTEGER NOT NULL,
    migration_name   TEXT NOT NULL,
    applied_on       INTEGER NOT NULL,
    UNIQUE (app_name, migration_number)
) STRICT;

"#;
