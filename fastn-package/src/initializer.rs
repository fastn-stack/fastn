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

#[async_trait::async_trait]
pub trait Initializer {
    async fn file_as_string(&self, path: &str) -> Result<String, FileAsStringError>;
}
