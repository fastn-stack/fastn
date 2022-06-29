#[derive(Debug, Clone)]
pub enum File {
    Ftd(Document),
    Static(Static),
    Markdown(Document),
    Code(Document),
    Image(Static),
}

impl File {
    pub fn get_id(&self) -> String {
        match self {
            Self::Ftd(a) => a.id.clone(),
            Self::Static(a) => a.id.clone(),
            Self::Markdown(a) => a.id.clone(),
            Self::Code(a) => a.id.clone(),
            Self::Image(a) => a.id.clone(),
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
    pub fn get_base_path(&self) -> String {
        match self {
            Self::Ftd(a) => a.parent_path.to_string(),
            Self::Static(a) => a.base_path.to_string(),
            Self::Markdown(a) => a.parent_path.to_string(),
            Self::Code(a) => a.parent_path.to_string(),
            Self::Image(a) => a.base_path.to_string(),
        }
    }
    pub fn get_full_path(&self) -> camino::Utf8PathBuf {
        let (id, base_path) = match self {
            Self::Ftd(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Static(a) => (a.id.to_string(), a.base_path.to_string()),
            Self::Markdown(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Code(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Image(a) => (a.id.to_string(), a.base_path.to_string()),
        };
        camino::Utf8PathBuf::from(base_path).join(id)
    }

    pub(crate) fn get_content(&self) -> Vec<u8> {
        match self {
            Self::Ftd(ref a) => a.content.as_bytes().to_vec(),
            Self::Static(ref a) => a.content.clone(),
            Self::Markdown(ref a) => a.content.as_bytes().to_vec(),
            Self::Code(ref a) => a.content.as_bytes().to_vec(),
            Self::Image(ref a) => a.content.clone(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct Document {
    pub package_name: String,
    pub id: String,
    pub content: String,
    pub parent_path: String,
}

impl Document {
    pub fn id_to_path(&self) -> String {
        fpm::utils::id_to_path(self.id.as_str())
    }

    pub fn id_with_package(&self) -> String {
        format!(
            "{}/{}",
            self.package_name.as_str(),
            self.id
                .replace("/index.ftd", "/")
                .replace("index.ftd", "")
                .replace(".ftd", "/")
        )
    }
}

#[derive(Debug, Clone)]
pub struct Static {
    pub id: String,
    pub content: Vec<u8>,
    pub base_path: camino::Utf8PathBuf,
}

pub(crate) async fn paths_to_files(
    package_name: &str,
    files: Vec<camino::Utf8PathBuf>,
    base_path: &camino::Utf8Path,
) -> fpm::Result<Vec<fpm::File>> {
    let pkg = package_name.to_string();
    Ok(futures::future::join_all(
        files
            .into_iter()
            .map(|x| {
                let base = base_path.to_path_buf();
                let p = pkg.clone();
                tokio::spawn(async move { fpm::get_file(p, &x, &base).await })
            })
            .collect::<Vec<tokio::task::JoinHandle<fpm::Result<fpm::File>>>>(),
    )
    .await
    .into_iter()
    .flatten()
    .flatten()
    .collect::<Vec<fpm::File>>())
}

pub fn package_ignores(
    package: &fpm::Package,
    root_path: &camino::Utf8PathBuf,
    ignore_history: bool,
) -> Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new(root_path);
    if ignore_history {
        overrides.add("!.history")?;
    }
    overrides.add("!.packages")?;
    overrides.add("!.tracks")?;
    overrides.add("!FPM")?;
    overrides.add("!rust-toolchain")?;
    overrides.add("!.build")?;
    for ignored_path in &package.ignored_paths {
        overrides.add(format!("!{}", ignored_path).as_str())?;
    }
    overrides.build()
}

pub(crate) async fn get_file(
    package_name: String,
    doc_path: &camino::Utf8Path,
    base_path: &camino::Utf8Path,
) -> fpm::Result<File> {
    if doc_path.is_dir() {
        return Err(fpm::Error::UsageError {
            message: format!("{} should be a file", doc_path.as_str()),
        });
    }

    let id = match std::fs::canonicalize(doc_path)?
        .to_str()
        .unwrap()
        .rsplit_once(
            if base_path.as_str().ends_with(std::path::MAIN_SEPARATOR) {
                base_path.as_str().to_string()
            } else {
                format!("{}{}", base_path, std::path::MAIN_SEPARATOR)
            }
            .as_str(),
        ) {
        Some((_, id)) => id.to_string(),
        None => {
            return Err(fpm::Error::UsageError {
                message: format!("{:?} should be a file", doc_path),
            });
        }
    };

    Ok(match id.rsplit_once(".") {
        Some((_, "ftd")) => File::Ftd(Document {
            package_name: package_name.to_string(),
            id: id.to_string(),
            content: tokio::fs::read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
        }),
        Some((_, "md")) => File::Markdown(Document {
            package_name: package_name.to_string(),
            id: id.to_string(),
            content: tokio::fs::read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
        }),
        Some((_, ext))
            if mime_guess::MimeGuess::from_ext(ext)
                .first_or_octet_stream()
                .to_string()
                .starts_with("image/") =>
        {
            File::Image(Static {
                id: id.to_string(),
                content: tokio::fs::read(&doc_path).await?,
                base_path: base_path.to_path_buf(),
            })
        }
        Some((_, ext)) if ftd::render::KNOWN_EXTENSIONS.contains(ext) => File::Code(Document {
            package_name: package_name.to_string(),
            id: id.to_string(),
            content: tokio::fs::read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
        }),
        _ => File::Static(Static {
            id: id.to_string(),
            content: tokio::fs::read(&doc_path).await?,
            base_path: base_path.to_path_buf(),
        }),
    })
}
