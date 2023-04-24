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
    fastn_package::sqlite::initialize_db()?;
    process_fastn_ftd(i).await?;
    todo!()
}

#[derive(thiserror::Error, Debug)]
pub enum FastnFTDError {
    #[error("fastn.ftd error: {source}")]
    CantReadFTDFile {
        #[from]
        source: fastn_package::initializer::FileAsStringError,
    },
}

async fn process_fastn_ftd(
    i: impl fastn_package::initializer::Initializer,
) -> Result<(), FastnFTDError> {
    let _content = i.file_as_string("FASTN.ftd").await?;

    todo!()
}
