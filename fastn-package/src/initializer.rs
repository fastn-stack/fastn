#[async_trait::async_trait]
pub trait Initializer {
    async fn file_as_string(
        &self,
        path: &str,
    ) -> Result<String, fastn_issues::initialization::FileAsStringError>;
}

pub mod test {
    pub use super::Initializer;
    pub use fastn_issues::initialization::{
        FastnFTDError, FileAsStringError, InitializePackageError,
    };

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
        async fn file_as_string(
            &self,
            path: &str,
        ) -> Result<String, fastn_issues::initialization::FileAsStringError> {
            let content = self.files.get(path).ok_or_else(|| {
                fastn_issues::initialization::FileAsStringError::FileDoesNotExist {
                    name: path.to_string(),
                    source: std::io::Error::from(std::io::ErrorKind::NotFound),
                }
            })?;
            Ok(content.to_string())
        }
    }
}
