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

pub mod test {
    pub use super::FileAsStringError;
    pub use super::Initializer;

    pub use fastn_package::initialize::{FastnFTDError, InitialisePackageError};

    pub struct TestInitializer {
        files: std::collections::HashMap<String, String>,
    }

    impl TestInitializer {
        pub fn new() -> Self {
            Self {
                files: std::collections::HashMap::new(),
            }
        }

        pub fn add_file(&mut self, name: &str, content: &str) {
            self.files.insert(name.to_string(), content.to_string());
        }
    }

    #[async_trait::async_trait]
    impl Initializer for TestInitializer {
        async fn file_as_string(&self, path: &str) -> Result<String, FileAsStringError> {
            let content =
                self.files
                    .get(path)
                    .ok_or_else(|| FileAsStringError::FileDoesNotExist {
                        name: path.to_string(),
                        source: std::io::Error::from(std::io::ErrorKind::NotFound),
                    })?;
            Ok(content.to_string())
        }
    }
}
