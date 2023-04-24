#[derive(thiserror::Error, Debug)]
pub enum InitialisePackageError {
    #[error("fastn.ftd error: {source}")]
    FastnFTDError {
        #[from]
        source: FastnFTDError,
    },
}

pub async fn initialize_packages(
    i: impl fastn_package::Initializer,
) -> Result<(), InitialisePackageError> {
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

async fn process_fastn_ftd(_i: impl fastn_package::Initializer) -> Result<(), FastnFTDError> {
    let content = i.read_file("FASTN.ftd").await?;

    todo!()
}
