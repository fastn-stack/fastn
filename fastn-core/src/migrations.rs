const MIGRATION_TABLE: &str = r#"

CREATE TABLE IF NOT EXISTS
    fastn_migration
(
    id               INTEGER PRIMARY KEY,
    app_name         TEXT NOT NULL,
    migration_number INTEGER NOT NULL UNIQUE,
    migration_name   TEXT NOT NULL,
    applied_on       INTEGER NOT NULL
) STRICT;

"#;

pub(crate) async fn migrate(config: &fastn_core::Config) -> Result<(), MigrationError> {
    // If there are no migrations, exit early.
    if !has_migrations(config) {
        return Ok(());
    }

    create_migration_table(config).await?;

    let latest_applied_migration_number = find_latest_applied_migration_number(config).await?;
    let migrations = find_migrations_to_apply(config, latest_applied_migration_number)?;

    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    for migration in migrations {
        println!("Applying Migration: {}", migration.name);
        apply_migration(config, &migration, now).await?;
    }

    Ok(())
}

async fn apply_migration(
    config: &fastn_core::Config,
    migration: &fastn_core::package::MigrationData,
    now: i64,
) -> Result<(), MigrationError> {
    let db = config.get_db_url().await;
    validate_migration(&migration)?;
    // Create the SQL to mark the migration as applied.
    let mark_migration_applied_content = mark_migration_applied_content(&config, &migration, now);

    // Combine the user-provided migration content and the marking content to run in a
    // transaction.
    let migration_content = format!(
        "{}\n\n{}",
        migration.content, mark_migration_applied_content
    );

    config
        .ds
        .sql_batch(db.as_str(), migration_content.as_str())
        .await?;

    Ok(())
}

fn find_migrations_to_apply(
    config: &fastn_core::Config,
    after: Option<i64>,
) -> Result<Vec<fastn_core::package::MigrationData>, MigrationError> {
    let mut migrations = vec![];

    for migration in config.package.migrations.iter() {
        if Some(migration.number) > after {
            migrations.push(migration.clone())
        }
    }

    Ok(migrations)
}

fn validate_migration(
    migration: &fastn_core::package::MigrationData,
) -> Result<(), MigrationError> {
    // Check for alphanumeric characters for migration name
    let alphanumeric_regex = regex::Regex::new(r"^[a-zA-Z0-9_-]+$").unwrap();
    if !alphanumeric_regex.is_match(&migration.name) {
        return Err(MigrationError::InvalidMigrationName {
            name: migration.name.to_string(),
        });
    }
    Ok(())
}

fn has_migrations(config: &fastn_core::Config) -> bool {
    !config.package.migrations.is_empty()
}

async fn create_migration_table(config: &fastn_core::Config) -> Result<(), fastn_utils::SqlError> {
    let db = config.get_db_url().await;

    config.ds.sql_batch(&db, MIGRATION_TABLE).await?;
    Ok(())
}

async fn find_latest_applied_migration_number(
    config: &fastn_core::Config,
) -> Result<Option<i64>, MigrationError> {
    let db = config.get_db_url().await;

    let results = config
        .ds
        .sql_query(
            db.as_str(),
            format!(
                r#"
                    SELECT
                        migration_number
                    FROM
                        fastn_migration
                    WHERE
                        app_name = '{}'
                    ORDER BY migration_number DESC
                    LIMIT 1;
                "#,
                config.package.name
            )
            .as_str(),
            vec![],
        )
        .await?;

    match results.len() {
        0 => Ok(None),
        1 => Ok(Some(
            serde_json::from_value::<i64>(results[0].get(0).unwrap().clone()).unwrap(),
        )), // Unwrap is okay here
        _ => unreachable!(),
    }
}

fn mark_migration_applied_content(
    config: &fastn_core::Config,
    migration_data: &fastn_core::package::MigrationData,
    now: i64,
) -> String {
    format!(
        r#"
            INSERT INTO
                fastn_migration
                    (app_name, migration_number, migration_name, applied_on)
            VALUES
                ('{}', {}, '{}', {});
        "#,
        config.package.name, migration_data.number, migration_data.name, now
    )
}
#[derive(thiserror::Error, Debug)]
pub enum MigrationError {
    #[error("Sql Error: {0}")]
    SqlError(#[from] fastn_utils::SqlError),
    #[error("Cannot delete applied migration")]
    AppliedMigrationDeletion,
    #[error("The migration order has changed or its content has been altered")]
    AppliedMigrationMismatch,
    #[error("Multiple migrations found with the same name: {name}.")]
    MigrationNameConflict { name: String },
    #[error("`{name}` is invalid migration name. It must contain only alphanumeric characters, underscores, and hyphens.")]
    InvalidMigrationName { name: String },
}
