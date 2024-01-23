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

static MIGRATIONS: diesel_async_migrations::EmbeddedMigrations =
    diesel_async_migrations::embed_migrations!();

/// run migrations on `db_url`
pub async fn migrate(db_url: impl AsRef<str>) -> fastn_core::Result<()> {
    use diesel_async::AsyncConnection;

    let mut conn = diesel_async::AsyncPgConnection::establish(db_url.as_ref())
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to connect to db. {:?}", e),
        })?;

    MIGRATIONS
        .run_pending_migrations(&mut conn)
        .await
        .map_err(|e| fastn_core::Error::DatabaseError {
            message: format!("Failed to run migrations. {:?}", e),
        })?;

    Ok(())
}
