mod fastn_migrations;

pub(crate) async fn migrate(config: &fastn_core::Config) -> Result<(), MigrationError> {
    // If there are no migrations, exit early.
    if !has_migrations(config) {
        return Ok(());
    }

    create_migration_table(config).await?;

    let now = chrono::Utc::now().timestamp_nanos_opt().unwrap();
    migrate_fastn(config, now).await?;
    migrate_app(config, now).await?;

    Ok(())
}

async fn migrate_app(config: &fastn_core::Config, now: i64) -> Result<(), MigrationError> {
    if !config.package.migrations.is_empty() {
        migrate_(
            config,
            config.package.migrations.as_slice(),
            config.package.name.as_str(),
            now,
        )
        .await?;
    }

    for app in config.package.apps.iter() {
        migrate_(
            config,
            app.package.migrations.as_slice(),
            app.name.as_str(),
            now,
        )
        .await?;
    }

    Ok(())
}

async fn migrate_fastn(config: &fastn_core::Config, now: i64) -> Result<(), MigrationError> {
    migrate_(
        config,
        fastn_migrations::fastn_migrations().as_slice(),
        "fastn",
        now,
    )
    .await
}

async fn migrate_(
    config: &fastn_core::Config,
    available_migrations: &[fastn_core::package::MigrationData],
    app_name: &str,
    now: i64,
) -> Result<(), MigrationError> {
    let latest_applied_migration_number =
        find_latest_applied_migration_number(config, app_name).await?;
    let migrations =
        find_migrations_to_apply(available_migrations, latest_applied_migration_number)?;

    for migration in migrations {
        println!("Applying Migration for {app_name}: {}", migration.name);
        apply_migration(config, app_name, &migration, now).await?;
    }

    Ok(())
}

async fn apply_migration(
    config: &fastn_core::Config,
    app_name: &str,
    migration: &fastn_core::package::MigrationData,
    now: i64,
) -> Result<(), MigrationError> {
    let db = config.get_db_url().await;
    validate_migration(migration)?;
    // Create the SQL to mark the migration as applied.
    let mark_migration_applied_content = mark_migration_applied_content(app_name, migration, now);

    // Combine the user-provided migration content and the marking content to run in a
    // transaction.
    let migration_content = format!(
        "BEGIN;\n{}\n\n{}\nCOMMIT;",
        migration.content, mark_migration_applied_content
    );

    config
        .ds
        .sql_batch(db.as_str(), migration_content.as_str())
        .await?;

    Ok(())
}

fn find_migrations_to_apply(
    available_migrations: &[fastn_core::package::MigrationData],
    after: Option<i64>,
) -> Result<Vec<fastn_core::package::MigrationData>, MigrationError> {
    let mut migrations = vec![];

    for migration in available_migrations.iter() {
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
        || config
            .package
            .apps
            .iter()
            .any(|a| !a.package.migrations.is_empty())
}

async fn create_migration_table(config: &fastn_core::Config) -> Result<(), fastn_utils::SqlError> {
    let db = config.get_db_url().await;

    config
        .ds
        .sql_batch(&db, fastn_migrations::MIGRATION_TABLE)
        .await?;
    Ok(())
}

async fn find_latest_applied_migration_number(
    config: &fastn_core::Config,
    app_name: &str,
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
                        app_name = '{app_name}'
                    ORDER BY migration_number DESC
                    LIMIT 1;
                "#,
            )
            .as_str(),
            &[],
        )
        .await?;

    match results.len() {
        0 => Ok(None),
        1 => Ok(Some(
            serde_json::from_value::<i64>(results[0].first().unwrap().clone()).unwrap(),
        )), // Unwrap is okay here
        _ => unreachable!(),
    }
}

fn mark_migration_applied_content(
    app_name: &str,
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
        app_name, migration_data.number, migration_data.name, now
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
