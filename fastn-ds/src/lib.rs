#[derive(Debug, Clone)]
pub struct DocumentStore {
    root: Path,
}

#[derive(Debug, Clone)]
pub struct Path {
    path: camino::Utf8PathBuf,
}

impl Path {
    pub fn new<T: AsRef<str>>(path: T) -> Self {
        Self {
            path: camino::Utf8PathBuf::from(path.as_ref()),
        }
    }

    pub fn join<T: AsRef<str>>(&self, path: T) -> Self {
        Self {
            path: self.path.join(path.as_ref()),
        }
    }
}

#[derive(thiserror::Error, Debug)]
pub enum ReadError {
    #[error("io error {0}")]
    IOError(#[from] std::io::Error),
    #[error("error")]
    Error,
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
    pub fn new<T: AsRef<camino::Utf8Path>>(root: T) -> Self {
        Self {
            root: root.as_ref().into(),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub async fn read_content(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(self.root.join(&path.path)).await?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        Ok(contents)
    }

    pub async fn read_to_string(&self, path: &Path) -> Result<String, ReadStringError> {
        self.read_content(path)
            .await
            .map_err(ReadStringError::ReadError)
            .and_then(|v| String::from_utf8(v).map_err(ReadStringError::UTF8Error))
    }

    pub async fn write_content(&self, path: &Path, data: Vec<u8>) -> Result<(), WriteError> {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::File::create(self.root.join(&path.path)).await?;
        file.write_all(&data).await?;
        Ok(())
    }

    pub async fn read_dir(&self, path: &Path) -> std::io::Result<tokio::fs::ReadDir> {
        // Todo: Return type should be ftd::interpreter::Result<Vec<fastn_ds::Dir>> not ftd::interpreter::Result<tokio::fs::ReadDir>
        tokio::fs::read_dir(&path.path).await
    }
}
