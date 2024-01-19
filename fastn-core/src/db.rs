pub type PgPool = diesel_async::pooled_connection::deadpool::Pool<diesel_async::AsyncPgConnection>;

async fn create_pool() -> fastn_core::Result<PgPool> {
    let db_url = std::env::var("FASTN_DB_URL")?;

    let config = diesel_async::pooled_connection::AsyncDieselConnectionManager::new(db_url);

    PgPool::builder(config)
        .build()
        .map_err(|e| fastn_core::error::Error::generic(format!("Failed to build db pool: {e}")))
}

static POOL_RESULT: tokio::sync::OnceCell<fastn_core::Result<PgPool>> =
    tokio::sync::OnceCell::const_new();

pub async fn pool() -> &'static fastn_core::Result<PgPool> {
    POOL_RESULT.get_or_init(create_pool).await
}

/// run migrations on `db_url`
pub fn migrate(db_url: String) -> fastn_core::Result<()> {
    use diesel::Connection;
    use diesel_migrations::MigrationHarness;

    const MIGRATIONS: diesel_migrations::EmbeddedMigrations =
        diesel_migrations::embed_migrations!();

    let mut conn = diesel::pg::PgConnection::establish(&db_url).map_err(|e| {
        fastn_core::Error::DatabaseError {
            message: format!("Failed to connect to db. {:?}", e),
        }
    })?;

    conn.run_pending_migrations(MIGRATIONS)
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to run migrations. {:?}", e),
        })?;

    Ok(())
}
