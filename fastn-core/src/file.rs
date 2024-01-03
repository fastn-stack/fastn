#[derive(Debug, Clone)]
pub enum File {
    Ftd(Document),
    Static(Static),
    Markdown(Document),
    Code(Document),
    Image(Static),
}

impl File {
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
    pub fn get_base_path(&self) -> &str {
        match self {
            Self::Ftd(a) => a.parent_path.as_str(),
            Self::Static(a) => a.base_path.as_str(),
            Self::Markdown(a) => a.parent_path.as_str(),
            Self::Code(a) => a.parent_path.as_str(),
            Self::Image(a) => a.base_path.as_str(),
        }
    }
    pub fn get_full_path(&self) -> fastn_ds::Path {
        let (id, base_path) = match self {
            Self::Ftd(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Static(a) => (a.id.to_string(), a.base_path.to_string()),
            Self::Markdown(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Code(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Image(a) => (a.id.to_string(), a.base_path.to_string()),
        };
        fastn_ds::Path::new(base_path).join(id)
    }

    pub(crate) fn get_content(&self) -> &[u8] {
        match self {
            Self::Ftd(ref a) => a.content.as_bytes(),
            Self::Static(ref a) => a.content.as_ref(),
            Self::Markdown(ref a) => a.content.as_bytes(),
            Self::Code(ref a) => a.content.as_bytes(),
            Self::Image(ref a) => a.content.as_ref(),
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
}

pub(crate) async fn paths_to_files(
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

pub fn package_ignores(
    package: &fastn_core::Package,
    root_path: &fastn_ds::Path,
) -> Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new(root_path);
    overrides.add("!.history")?;
    overrides.add("!.packages")?;
    overrides.add("!.tracks")?;
    overrides.add("!fastn")?;
    overrides.add("!rust-toolchain")?;
    overrides.add("!.build")?;
    overrides.add("!_tests")?;
    for ignored_path in &package.ignored_paths {
        overrides.add(format!("!{}", ignored_path).as_str())?;
    }
    overrides.build()
}

pub fn ignore_path(
    package: &fastn_core::Package,
    root_path: &fastn_ds::Path,
    ignore_paths: Vec<String>,
) -> Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new(root_path);
    overrides.add("!.packages")?;
    overrides.add("!.build")?;
    for ignored_path in &package.ignored_paths {
        overrides.add(format!("!{}", ignored_path).as_str())?;
    }
    for ignored_path in ignore_paths {
        overrides.add(format!("!{}", ignored_path).as_str())?;
    }
    overrides.build()
}

pub(crate) async fn get_file(
    ds: &fastn_ds::DocumentStore,
    package_name: String,
    doc_path: &fastn_ds::Path,
    base_path: &fastn_ds::Path,
) -> fastn_core::Result<File> {
    let base_path_str = base_path
        .to_string()
        .trim_end_matches(std::path::MAIN_SEPARATOR)
        .to_string();

    if !doc_path.to_string().starts_with(base_path_str) {
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
            content: ds.read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
        }),
        Some((_, "md")) => File::Markdown(Document {
            package_name: package_name.to_string(),
            id: id.to_string(),
            content: ds.read_to_string(&doc_path).await?,
            parent_path: base_path.to_string(),
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
                content: ds.read_content(&doc_path).await?,
                base_path: base_path.clone(),
            })
        }
        Some((_, ext)) if ftd::ftd2021::code::KNOWN_EXTENSIONS.contains(ext) => {
            File::Code(Document {
                package_name: package_name.to_string(),
                id: id.to_string(),
                content: ds.read_to_string(&doc_path).await?,
                parent_path: base_path.to_string(),
            })
        }
        _ => File::Static(Static {
            package_name: package_name.to_string(),
            id: id.to_string(),
            content: ds.read_content(&doc_path).await?,
            base_path: base_path.clone(),
        }),
    })
}

pub fn is_static(path: &str) -> fastn_core::Result<bool> {
    Ok(match path.rsplit_once('.') {
        Some((_, "ftd")) | Some((_, "md")) => false,
        Some((_, "svg")) | Some((_, "woff")) | Some((_, "woff2")) => true,
        Some((_, ext)) if ftd::ftd2021::code::KNOWN_EXTENSIONS.contains(ext) => false,
        None => false,
        _ => true,
    })
}
