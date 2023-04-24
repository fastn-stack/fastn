#[derive(thiserror::Error, Debug)]
pub enum FileAsStringError {}

#[async_trait::async_trait]
pub trait Initializer {
    async fn file_as_string(&self, path: &str) -> Result<String, FileAsStringError>;
}
