#[derive(Debug, Clone)]
pub struct DocumentStore {
    pub root: std::path::PathBuf,
}

impl DocumentStore {
    pub fn new<T: AsRef<std::path::Path>>(root: T) -> Self {
        Self {
            root: root.as_ref().into(),
        }
    }

    pub async fn read_content<T: AsRef<str>>(&self, path: T) -> ftd::interpreter::Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(self.root.join(path.as_ref())).await?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        Ok(contents)
    }

    pub async fn read_to_string<T: AsRef<str>>(&self, path: T) -> ftd::interpreter::Result<String> {
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
        data: &[u8],
    ) -> ftd::interpreter::Result<()> {
        use tokio::io::AsyncWriteExt;
        let full_path = dbg!(self.root.join(path.as_ref()));
        if let Some(parent) = full_path.parent() {
            tokio::fs::create_dir_all(parent).await?;
        }
        let mut file = tokio::fs::File::create(full_path).await?;
        file.write_all(data).await?;
        Ok(())
    }

    pub async fn read_dir<T: AsRef<str>>(
        &self,
        path: T,
    ) -> ftd::interpreter::Result<tokio::fs::ReadDir> {
        // Todo: Return type should be ftd::interpreter::Result<Vec<fastn_ds::Dir>> not ftd::interpreter::Result<tokio::fs::ReadDir>
        Ok(tokio::fs::read_dir(path.as_ref()).await?)
    }
}
