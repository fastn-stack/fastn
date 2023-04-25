/// re_initialise() is called when any file is changed in the package
pub async fn re_initialise(
    i: impl fastn_package::initializer::Initializer,
) -> Result<(), fastn_issues::initialization::InitializePackageError> {
    if let Some(v) = fastn_package::FTD_CACHE.get() {
        let mut v = v.write().await;
        v.clear();
    }
    let conn = fastn_package::sqlite::initialize_db()?;
    process_fastn_ftd(i, conn).await?;
    todo!()
}

/// initialise() is called on application start
pub async fn initialize(
    i: impl fastn_package::initializer::Initializer,
) -> Result<(), fastn_issues::initialization::InitializePackageError> {
    fastn_package::FTD_CACHE
        .get_or_init(|| async { tokio::sync::RwLock::new(std::collections::HashMap::new()) })
        .await;
    re_initialise(i).await
}

async fn process_fastn_ftd(
    i: impl fastn_package::initializer::Initializer,
    conn: rusqlite::Connection,
) -> Result<(), fastn_issues::initialization::FastnFTDError> {
    let content = i.file_as_string("FASTN.ftd").await?;
    let fastn_doc = fastn_package::old_fastn::parse_old_fastn(content.as_str())?;
    store_name(conn, fastn_doc).await?;

    todo!()
}

async fn store_name(
    _conn: rusqlite::Connection,
    fastn_doc: ftd::ftd2021::p2::Document,
) -> Result<(), fastn_issues::initialization::StoreNameError> {
    let _name = fastn_package::old_fastn::get_name(fastn_doc)?;
    // TODO: insert package name to main_package table
    todo!()
}
