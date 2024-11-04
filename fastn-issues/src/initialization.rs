#[derive(thiserror::Error, Debug)]
pub enum InitializePackageError {
    #[error("fastn.ftd error: {source}")]
    FastnFTDError {
        #[from]
        source: FastnFTDError,
    },
    #[error("db initialisation error: {source}")]
    InitializeDBError {
        #[from]
        source: InitializeDBError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum FastnFTDError {
    #[error("Can't read FASTN.ftd: {source}")]
    ReadFTDFile {
        #[from]
        source: FileAsStringError,
    },
    #[error("Cant parse FASTN.ftd: {source}")]
    ParseFASTNFile {
        #[from]
        source: OldFastnParseError,
    },
    #[error("Cant store package name: {source}")]
    StorePackageName {
        #[from]
        source: StoreNameError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum StoreNameError {
    #[error("Cant get package name from FASTN.ftd: {source}")]
    CantGetPackageName {
        #[from]
        source: GetNameError,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum FileAsStringError {
    #[error("file not found: {name}, {source}")]
    FileDoesNotExist {
        name: String,
        source: std::io::Error,
    },
    #[error("file not found: {name}, {source}")]
    PathIsNotAFile {
        name: String,
        source: std::io::Error,
    },
    #[error("file not found: {name}, {source}")]
    CantReadFile {
        name: String,
        source: std::io::Error,
    },
    #[error("file not found: {name}, {source}")]
    ContentIsNotUTF8 {
        name: String,
        source: std::io::Error,
    },
}

#[derive(thiserror::Error, Debug)]
pub enum OldFastnParseError {
    #[error("FASTN.ftd is invalid ftd: {source}")]
    FTDError {
        #[from]
        source: ftd::ftd2021::p1::Error,
    },
    #[error("FASTN.ftd imported something other then fastn: {module}")]
    InvalidImport { module: String },
    #[error("FASTN.ftd tried to use a processor: {processor}")]
    ProcessorUsed { processor: String },
}

#[derive(thiserror::Error, Debug)]
pub enum GetNameError {
    #[error("Can't find fastn.package in FASTN.ftd, this is impossible: {source}")]
    CantFindPackage {
        #[from]
        source: ftd::ftd2021::p1::Error,
    },
    #[error("fastn.package was not initialised in FASTN.ftd")]
    PackageIsNone,
}

#[derive(thiserror::Error, Debug)]
pub enum InitializeDBError {
    #[error("cant open db connection: {source}")]
    OpenDBConnection { source: rusqlite::Error },
    #[error("cant create tables: {source}")]
    CreateTables { source: rusqlite::Error },
}
