#[derive(thiserror::Error, Debug)]
pub enum InitialisePackageError {
    #[error("fastn.ftd error: {source}")]
    FastnFTDError {
        #[from]
        source: FastnFTDError,
    },
    #[error("db initialisation error: {source}")]
    InitializeDBError {
        #[from]
        source: fastn_package::sqlite::InitializeDBError,
    },
}

pub async fn initialize(
    i: impl fastn_package::initializer::Initializer,
) -> Result<(), InitialisePackageError> {
    fastn_package::FTD_CACHE
        .get_or_init(|| async { tokio::sync::RwLock::new(std::collections::HashMap::new()) })
        .await;
    let conn = fastn_package::sqlite::initialize_db()?;
    process_fastn_ftd(i, conn).await?;
    todo!()
}

#[derive(thiserror::Error, Debug)]
pub enum FastnFTDError {
    #[error("Can't read FASTN.ftd: {source}")]
    CantReadFTDFile {
        #[from]
        source: fastn_package::initializer::FileAsStringError,
    },
    #[error("Cant parse FASTN.ftd: {source}")]
    CantParseFASTNFile {
        #[from]
        source: fastn_package::old_fastn::OldFastnParseError,
    },
    #[error("Cant get package name from FASTN.ftd: {source}")]
    CantGetPackageName {
        #[from]
        source: fastn_package::old_fastn::GetNameError,
    },
}

async fn process_fastn_ftd(
    i: impl fastn_package::initializer::Initializer,
    _conn: rusqlite::Connection,
) -> Result<(), FastnFTDError> {
    let content = i.file_as_string("FASTN.ftd").await?;
    let doc = fastn_package::old_fastn::parse_old_fastn(content.as_str())?;
    // TODO: insert package name to main_package table
    let _name = fastn_package::old_fastn::get_name(doc)?;
    todo!()
}
