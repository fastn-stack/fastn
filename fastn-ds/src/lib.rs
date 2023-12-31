#[derive(Debug, Clone)]
pub struct DocumentStore {
    pub root: std::path::PathBuf,
}

impl DocumentStore {
    pub fn new<T: AsRef<str>>(root: T) -> Self {
        Self {
            root: root.as_ref().into(),
        }
    }

    pub async fn read_content<T: AsRef<str>>(
        &self,
        path: T,
        _user_id: Option<u32>,
    ) -> ftd::interpreter::Result<Vec<u8>> {
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(self.root.join(path.as_ref())).await?;
        let mut contents = vec![];
        file.read_to_end(&mut contents).await?;
        Ok(contents)
    }

    pub async fn read_to_string<T: AsRef<str>>(
        &self,
        path: T,
        _user_id: Option<u32>,
    ) -> ftd::interpreter::Result<String> {
        use tokio::io::AsyncReadExt;

        let path = path.as_ref();
        let mut file = tokio::fs::File::open(self.root.join(path)).await?;
        let mut contents = String::new();
        file.read_to_string(&mut contents).await?;
        Ok(contents)
    }

    pub async fn write_content(
        &self,
        path: &str,
        data: &[u8],
        _user_id: Option<u32>,
    ) -> ftd::interpreter::Result<()> {
        use tokio::io::AsyncWriteExt;

        let mut file = tokio::fs::File::create(self.root.join(path)).await?;
        file.write_all(data).await?;
        Ok(())
    }
}
