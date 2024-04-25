#[derive(thiserror::Error, Debug)]
pub enum MigrationError {
    #[error("pool error: {0}")]
    PoolError(#[from] deadpool::managed::PoolError<tokio_postgres::Error>),

    #[error("pg error {0}")]
    Pg(#[from] tokio_postgres::Error),

    #[error("unknown database")]
    UnknownDatabase,
}

pub async fn migrate(
    pg_pool: &deadpool_postgres::Pool,
    sqlite: actix_web::web::Data<async_lock::Mutex<Option<rusqlite::Connection>>>,
    db_url: &str,
) -> Result<(), MigrationError> {
    if db_url.starts_with("sqlite") {
        migrate_sqlite(sqlite).await
    } else if db_url.starts_with("postgresql") {
        migrate_pg(pg_pool).await
    } else {
        Err(MigrationError::UnknownDatabase)
    }
}

pub async fn migrate_sqlite(
    _sqlite: actix_web::web::Data<async_lock::Mutex<Option<rusqlite::Connection>>>,
) -> Result<(), MigrationError> {
    todo!()
}
async fn migrate_pg(pg_pool: &deadpool_postgres::Pool) -> Result<(), MigrationError> {
    let client = pg_pool.get().await?;
    client
        .batch_execute(include_str!("../user-core-pg.sql"))
        .await?;
    // TODO: get list of migrations from all dependencies and see if any of them needs to
    //       be applied
    Ok(())
}
