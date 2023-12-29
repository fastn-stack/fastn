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
