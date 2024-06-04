pub(super) const MIGRATION_TABLE: &str = r#"

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

pub(crate) async fn create_migration_table(
    config: &fastn_core::Config,
    db: &str,
) -> Result<(), fastn_utils::SqlError> {
    config.ds.sql_batch(db, MIGRATION_TABLE).await?;
    Ok(())
}

pub(crate) async fn migrate(
    req_config: &mut fastn_core::RequestConfig,
    db: &str,
) -> Result<(), fastn_utils::SqlError> {
    let migration_ftd = get_migrations_from_ftd(req_config).await;
    let migration_sql = get_migration_from_sql(&req_config.config, db).await?;

    let migrations_to_apply = find_migrations_to_apply(&migration_ftd, &migration_sql).await;

    let now = chrono::Utc::now();
    for migration in migrations_to_apply {
        req_config
            .config
            .ds
            .sql_batch(db, migration.content.as_str())
            .await?;
        mark_migration_applied(&req_config.config, db, &migration, &now)
    }
    Ok(())
}

async fn mark_migration_applied(
    config: &fastn_core::Config,
    db: &str,
    migration_data: &MigrationDataFTD,
    now: &chrono::DateTime<chrono::Utc>,
) -> Result<(), fastn_utils::SqlError> {
    let now_in_nanosecond = now.timestamp_nanos_opt().unwrap();
    config
        .ds
        .sql_execute(
            db,
            format!(
                r#"
                    INSERT INTO
                        fastn_migration
                            (app_name, migration_number, migration_name, applied_on)
                    VALUES
                        ({}, {}, {}, {});
                "#,
                config.package.name, migration_data.number, migration_data.name, now_in_nanosecond
            )
            .as_str(),
            vec![],
        )
        .await?;
    Ok(())
}

struct MigrationDataFTD {
    number: i64,
    name: String,
    content: String,
}

async fn get_migrations_from_ftd(
    req_config: &mut fastn_core::RequestConfig,
) -> Vec<MigrationDataFTD> {
    let migration_ftd = req_config
        .config
        .package
        .fs_fetch_by_file_name("migration.ftd", None, &req_config.config.ds)
        .await?;
    let migration_ftd_str = String::from_utf8(migration_ftd).unwrap();

    let main_ftd_doc = fastn_core::doc::interpret_helper(
        "migration",
        migration_ftd_str.as_str(),
        req_config,
        "/",
        false,
        0,
    )
    .await?;
    let mut migration_data = vec![];
    let mut bag = main_ftd_doc.data.clone();
    bag.extend(ftd::interpreter::default::default_migration_bag());

    let doc = ftd::interpreter::TDoc::new(&main_ftd_doc.name, &main_ftd_doc.aliases, &bag);

    for (idx, component) in main_ftd_doc.tree.iter().enumerate() {
        if is_fastn_migration(component) {
            let property_values =
                component.get_interpreter_property_value_of_all_arguments(&doc)?;
            let query = property_values
                .get("query")
                .unwrap()
                .clone()
                .resolve(&doc, component.line_number)
                .unwrap()
                .to_json_string(&doc, false)?
                .unwrap();
            let migration_name = fastn_core::utils::generate_hash(&query);
            let migration_number = (idx + 1) as i64;
            migration_data.push(MigrationDataFTD {
                number: migration_number,
                name: migration_name,
                content: query,
            });
        } else {
            unimplemented!(); // todo: throw error
        }
    }

    migration_data
}

fn is_fastn_migration(component: &ftd::interpreter::Component) -> bool {
    component.name.eq("fastn#migration") || component.name.eq("fastn.migration")
}

struct MigrationDataSQL {
    number: i64,
    name: String,
}

async fn find_migrations_to_apply(
    migration_ftd: &[MigrationDataFTD],
    migration_sql: &[MigrationDataSQL],
) -> Vec<MigrationDataFTD> {
    if migration_sql.len() > migration_ftd.len() {
        unreachable!("Can't delete the applied migration") // todo: throw error
    }

    let migration_sql: std::collections::HashMap<i64, MigrationDataSQL> = migration_sql
        .into_iter()
        .map(|val| (*val.number, val.clone()))
        .collect();
    let mut migrations_to_apply = vec![];
    for m_ftd in migration_ftd {
        match migration_sql.get(*m_ftd.number) {
            Some(m_sql) => {
                if m_sql.name.ne(&m_ftd.name) {
                    unreachable!("Cannot change the content of migration") // todo: throw error
                }
            }
            None => {
                migrations_to_apply.push(m_ftd.clone());
            }
        }
    }
    migrations_to_apply
}

async fn get_migration_from_sql(
    config: &fastn_core::Config,
    db: &str,
) -> Result<Vec<MigrationDataSQL>, fastn_utils::SqlError> {
    let results = config
        .ds
        .sql_query(
            db,
            format!(
                r#"
                    SELECT
                        migration_number, migration_name
                    FROM
                        fastn_migration
                    WHERE
                        app_name='{}'
                "#,
                config.package.name
            ),
            vec![],
        )
        .await?;

    let mut migration_details = vec![];

    for entry in results {
        let migration_number: i64 = serde_json::from_value(
            entry
                .get(0)
                .expect("fastn_migration::migration_number not found.")
                .clone(),
        )
        .unwrap();
        let migration_name: String = serde_json::from_value(
            entry
                .get(1)
                .expect("fastn_migration::migration_name not found.")
                .clone(),
        )
        .unwrap();
        migration_details.push(MigrationDataSQL {
            number: migration_number,
            name: migration_name,
        });
    }
    Ok(migration_details)
}

pub(crate) async fn get_db_url(config: &fastn_core::Config) -> String {
    match config.ds.env("FASTN_DB_URL").await {
        Ok(db_url) => db_url,
        Err(_) => config
            .ds
            .env("DATABASE_URL")
            .await
            .unwrap_or_else(|_| "sqlite:///fastn.sqlite".to_string()),
    }
}
