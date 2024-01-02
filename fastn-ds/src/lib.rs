#[derive(Debug, Clone)]
pub struct DocumentStore {
    root: std::path::PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
    #[error("io error {0}")]
    IOError(#[from] std::io::Error),
}

#[derive(thiserror::Error, Debug)]
pub enum ReadStringError {
    #[error("read error {0}")]
    ReadError(#[from] ReadError),
    #[error("utf-8 error {0}")]
    UTF8Error(#[from] std::string::FromUtf8Error),
}

#[derive(thiserror::Error, Debug)]
pub enum WriteError {
    #[error("pool error {0}")]
    IOError(#[from] std::io::Error),
}

impl DocumentStore {
    pub fn new<T: AsRef<std::path::Path>>(root: T) -> Self {
        Self {
            root: root.as_ref().into(),
        }
    }

    pub fn root(&self) -> &std::path::Path {
        &self.root
    }

    pub async fn read_content<T: AsRef<str>>(&self, path: T) -> Result<Vec<u8>, ReadError> {
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(self.root.join(path.as_ref())).await?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        Ok(contents)
    }

    pub async fn read_to_string<T: AsRef<str>>(&self, path: T) -> Result<String, ReadStringError> {
        use tokio::io::AsyncReadExt;

        let path = path.as_ref();
        let mut file = tokio::fs::File::open(self.root.join(path)).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        Ok(contents)
    }

    pub async fn write_content<T: AsRef<str>>(
        &self,
        path: T,
        data: Vec<u8>,
    ) -> Result<(), WriteError> {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::File::create(self.root.join(path.as_ref())).await?;
        file.write_all(&data).await?;
        Ok(())
    }

    pub async fn read_dir<T: AsRef<str>>(&self, path: T) -> std::io::Result<tokio::fs::ReadDir> {
        // Todo: Return type should be ftd::interpreter::Result<Vec<fastn_ds::Dir>> not ftd::interpreter::Result<tokio::fs::ReadDir>
        Ok(tokio::fs::read_dir(path.as_ref()).await?)
    }
}
