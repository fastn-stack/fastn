#[derive(Debug, Clone)]
pub enum File {
    Ftd(Document),
    Static(Static),
    Markdown(Document),
    Code(Document),
    Image(Static),
}

impl File {
    pub fn content(&self) -> &[u8] {
        match self {
            Self::Ftd(a) => a.content.as_bytes(),
            Self::Static(a) => a.content.as_slice(),
            Self::Markdown(a) => a.content.as_bytes(),
            Self::Code(a) => a.content.as_bytes(),
            Self::Image(a) => a.content.as_slice(),
        }
    }

    pub fn get_id(&self) -> &str {
        match self {
            Self::Ftd(a) => a.id.as_str(),
            Self::Static(a) => a.id.as_str(),
            Self::Markdown(a) => a.id.as_str(),
            Self::Code(a) => a.id.as_str(),
            Self::Image(a) => a.id.as_str(),
        }
    }
    pub fn get_id_with_package(&self) -> String {
        match self {
            Self::Ftd(a) => a.id_with_package(),
            Self::Static(a) => a.id_with_package(),
            Self::Markdown(a) => a.id_with_package(),
            Self::Code(a) => a.id_with_package(),
            Self::Image(a) => a.id_with_package(),
        }
    }
    pub fn set_id(&mut self, new_id: &str) {
        *(match self {
            Self::Ftd(a) => &mut a.id,
            Self::Static(a) => &mut a.id,
            Self::Markdown(a) => &mut a.id,
            Self::Code(a) => &mut a.id,
            Self::Image(a) => &mut a.id,
        }) = new_id.to_string();
    }
    pub fn get_base_path(&self) -> &fastn_ds::Path {
        match self {
            Self::Ftd(a) => &a.parent_path,
            Self::Static(a) => &a.base_path,
            Self::Markdown(a) => &a.parent_path,
            Self::Code(a) => &a.parent_path,
            Self::Image(a) => &a.base_path,
        }
    }
    pub fn get_full_path(&self) -> fastn_ds::Path {
        match self {
            Self::Ftd(a) | Self::Markdown(a) | Self::Code(a) => a.get_full_path(),
            Self::Image(a) | Self::Static(a) => a.get_full_path(),
        }
    }

    pub fn is_static(&self) -> bool {
        matches!(
            self,
            Self::Image(_) | Self::Static(_) | Self::Markdown(_) | Self::Code(_)
        )
    }

    pub(crate) fn is_ftd(&self) -> bool {
        matches!(self, Self::Ftd(_))
    }

    pub(crate) fn get_ftd_document(self) -> Option<fastn_core::Document> {
        match self {
            fastn_core::File::Ftd(ftd_document) => Some(ftd_document),
            _ => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub package_name: String,
    pub id: String,
    pub content: String,
    pub parent_path: fastn_ds::Path,
}

impl Document {
    pub fn id_to_path(&self) -> String {
        fastn_core::utils::id_to_path(self.id.as_str())
    }

    pub fn id_with_package(&self) -> String {
        format!(
            "{}/{}",
            self.package_name,
            self.id
                .replace("/index.ftd", "/")
                .replace("index.ftd", "")
                .replace(".ftd", "/")
        )
    }

    pub fn get_full_path(&self) -> fastn_ds::Path {
        self.parent_path.join(self.id.as_str())
    }
}

#[derive(Debug, Clone)]
pub struct Static {
    pub package_name: String,
    pub id: String,
    pub content: Vec<u8>,
    pub base_path: fastn_ds::Path,
}

impl Static {
    pub fn id_with_package(&self) -> String {
        format!("{}/{}", self.package_name, self.id)
    }

    pub fn get_full_path(&self) -> fastn_ds::Path {
        self.base_path.join(self.id.as_str())
    }
}

pub async fn paths_to_files(
    ds: &fastn_ds::DocumentStore,
    package_name: &str,
    files: Vec<fastn_ds::Path>,
    base_path: &fastn_ds::Path,
) -> fastn_core::Result<Vec<fastn_core::File>> {
    let pkg = package_name.to_string();
    Ok(futures::future::join_all(
        files
            .into_iter()
            .map(|x| {
                let base = base_path.clone();
                let p = pkg.clone();
                let ds = ds.clone();
                tokio::spawn(async move { fastn_core::get_file(&ds, p, &x, &base).await })
            })
            .collect::<Vec<tokio::task::JoinHandle<fastn_core::Result<fastn_core::File>>>>(),
    )
    .await
    .into_iter()
    .flatten()
    .flatten()
    .collect::<Vec<fastn_core::File>>())
}

pub async fn get_file(
    ds: &fastn_ds::DocumentStore,
    package_name: String,
    doc_path: &fastn_ds::Path,
    base_path: &fastn_ds::Path,
) -> fastn_core::Result<File> {
    let base_path_str = base_path
        .to_string()
        .trim_end_matches(std::path::MAIN_SEPARATOR)
        .to_string();

    if !doc_path.to_string().starts_with(&base_path_str) {
        return Err(fastn_core::Error::UsageError {
            message: format!("{:?} should be a file", doc_path),
        });
    }

    let id = doc_path
        .to_string()
        .trim_start_matches(base_path_str.as_str())
        .replace(std::path::MAIN_SEPARATOR, "/")
        .trim_start_matches('/')
        .to_string();

    Ok(match id.rsplit_once('.') {
        Some((_, "ftd")) => File::Ftd(Document {
            package_name: package_name.to_string(),
            id: id.to_string(),
            content: ds.read_to_string(doc_path).await?,
            parent_path: base_path.clone(),
        }),
        Some((_, "md")) => File::Markdown(Document {
            package_name: package_name.to_string(),
            id: id.to_string(),
            content: ds.read_to_string(doc_path).await?,
            parent_path: base_path.clone(),
        }),
        Some((_, ext))
            if mime_guess::MimeGuess::from_ext(ext)
                .first_or_octet_stream()
                .to_string()
                .starts_with("image/") =>
        {
            File::Image(Static {
                package_name: package_name.to_string(),
                id: id.to_string(),
                content: ds.read_content(doc_path).await?,
                base_path: base_path.clone(),
            })
        }
        Some((_, ext)) if ftd::ftd2021::code::KNOWN_EXTENSIONS.contains(ext) => {
            File::Code(Document {
                package_name: package_name.to_string(),
                id: id.to_string(),
                content: ds.read_to_string(doc_path).await?,
                parent_path: base_path.clone(),
            })
        }
        _ => File::Static(Static {
            package_name: package_name.to_string(),
            id: id.to_string(),
            content: ds.read_content(doc_path).await?,
            base_path: base_path.clone(),
        }),
    })
}

pub fn is_static(path: &str) -> fastn_core::Result<bool> {
    Ok(
        match path
            .rsplit_once('/')
            .map(|v| v.1)
            .unwrap_or(path)
            .rsplit_once('.')
        {
            Some((_, "ftd")) | Some((_, "md")) => false,
            Some((_, "svg")) | Some((_, "woff")) | Some((_, "woff2")) => true,
            Some((_, ext)) if ftd::ftd2021::code::KNOWN_EXTENSIONS.contains(ext) => false,
            None => false,
            _ => true,
        },
    )
}
