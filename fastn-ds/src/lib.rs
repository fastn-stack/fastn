extern crate self as fastn_ds;

use std::fmt::Formatter;

#[derive(Debug, Clone)]
pub struct DocumentStore {
    root: Path,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Path {
    path: camino::Utf8PathBuf,
}

#[derive(thiserror::Error, Debug)]
pub enum RemoveError {
    #[error("io error {0}")]
    IOError(#[from] std::io::Error),
}

impl Path {
    pub fn new<T: AsRef<str>>(path: T) -> Self {
        Self {
            path: camino::Utf8PathBuf::from(path.as_ref()),
        }
    }

    pub fn to_string(&self) -> String {
        self.path.as_str().to_string()
    }

    pub fn join<T: AsRef<str>>(&self, path: T) -> Self {
        Self {
            path: self.path.join(path.as_ref()),
        }
    }

    pub fn parent(&self) -> Option<Self> {
        self.path.parent().map(|path| Path {
            path: path.to_path_buf(),
        })
    }

    pub fn strip_prefix(&self, base: &Self) -> Self {
        Path {
            path: self
                .path
                .strip_prefix(base.path.as_str())
                .unwrap()
                .to_path_buf(),
        }
    }

    pub fn get_all_file_path(&self, ignore_paths: &[String]) -> Vec<fastn_ds::Path> {
        let path = &self.path;
        let mut ignore_path = ignore::WalkBuilder::new(path);
        // ignore_paths.hidden(false); // Allow the linux hidden files to be evaluated
        ignore_path.overrides(package_ignores(ignore_paths, path).unwrap());
        ignore_path
            .build()
            .flatten()
            .map(|x| fastn_ds::Path {
                path: camino::Utf8PathBuf::from_path_buf(x.into_path()).unwrap(),
            }) //todo: improve error message
            .collect::<Vec<fastn_ds::Path>>()
    }

    pub fn exists(&self) -> bool {
        self.path.exists()
    }

    pub fn eq(&self, other: &Self) -> bool {
        self.path.eq(&other.path)
    }

    /// Remove path: It can be directory or file
    pub async fn remove(&self) -> Result<(), RemoveError> {
        if self.path.is_file() {
            tokio::fs::remove_file(&self.path).await?;
        } else if self.path.is_dir() {
            tokio::fs::remove_dir_all(&self.path).await?
        } else if self.path.is_symlink() {
            // TODO:
            // It can be a directory or a file
        }
        Ok(())
    }

    pub fn file_name(&self) -> Option<String> {
        self.path.file_name().map(|v| v.to_string())
    }
}

impl std::fmt::Display for fastn_ds::Path {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.path)
    }
}

fn package_ignores(
    ignore_paths: &[String],
    root_path: &camino::Utf8PathBuf,
) -> Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new(root_path);
    for ignored_path in ignore_paths {
        overrides.add(format!("!{}", ignored_path).as_str())?;
    }
    overrides.build()
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
            root: Path::new(root.as_ref().as_str()),
        }
    }

    pub fn root(&self) -> &Path {
        &self.root
    }

    pub async fn read_content(&self, path: &Path) -> Result<Vec<u8>, ReadError> {
        use tokio::io::AsyncReadExt;

        let mut file = tokio::fs::File::open(self.root.join(&path.path).path).await?;
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

        let full_path = self.root.join(&path.path);

        // Create the directory if it doesn't exist
        if let Some(parent) = full_path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent.path).await?;
            }
        }

        let mut file = tokio::fs::File::create(full_path.path).await?;
        file.write_all(&data).await?;
        Ok(())
    }

    pub async fn read_dir(&self, path: &Path) -> std::io::Result<tokio::fs::ReadDir> {
        // Todo: Return type should be ftd::interpreter::Result<Vec<fastn_ds::Dir>> not ftd::interpreter::Result<tokio::fs::ReadDir>
        tokio::fs::read_dir(&path.path).await
    }
}
