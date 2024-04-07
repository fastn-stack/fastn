#[derive(thiserror::Error, Debug)]
pub enum MigrationError {
    #[error("pool error: {0}")]
    PoolError(#[from] deadpool::managed::PoolError<tokio_postgres::Error>),

    #[error("pg error {0}")]
    Pg(#[from] tokio_postgres::Error),
}

pub async fn migrate(pool: &deadpool_postgres::Pool) -> Result<(), MigrationError> {
    let client = pool.get().await?;
    client
        .batch_execute(include_str!("../user-core.sql"))
        .await?;
    // TODO: get list of migrations from all dependencies and see if any of them needs to
    //       be applied
    Ok(())
}
