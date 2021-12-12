#[derive(Debug)]
pub enum File {
    Ftd(Document),
    Static(Static),
    Markdown(Document),
}

impl File {
    pub fn get_id(&self) -> String {
        match self {
            Self::Ftd(a) => a.id.clone(),
            Self::Static(a) => a.id.clone(),
            Self::Markdown(a) => a.id.clone(),
        }
    }
    pub fn get_base_path(&self) -> String {
        match self {
            Self::Ftd(a) => a.parent_path.to_string(),
            Self::Static(a) => a.base_path.to_string(),
            Self::Markdown(a) => a.parent_path.to_string(),
        }
    }
    pub fn get_full_path(&self) -> String {
        let (id, base_path) = match self {
            Self::Ftd(a) => (a.id.to_string(), a.parent_path.to_string()),
            Self::Static(a) => (a.id.to_string(), a.base_path.to_string()),
            Self::Markdown(a) => (a.id.to_string(), a.parent_path.to_string()),
        };
        format!("{}/{}", base_path, id)
    }
}

#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub content: String,
    pub parent_path: String,
    pub depth: usize,
}

#[derive(Debug)]
pub struct Static {
    pub id: String,
    pub base_path: String,
    pub depth: usize,
}

pub(crate) async fn get_documents(config: &fpm::Config) -> fpm::Result<Vec<File>> {
    let mut documents: Vec<File> = vec![];
    let mut ignore_paths = ignore::WalkBuilder::new("./");

    ignore_paths.overrides(package_ignores()?); // unwrap ok because this we know can never fail
    ignore_paths.standard_filters(true);
    ignore_paths.overrides(config.ignored.clone());
    // TODO: Get this concurrent async to work
    // let all_files = ignore_paths.build()
    //     .into_iter()
    //     .map(|x| {
    //         tokio::spawn(process_file_(
    //             &mut documents,
    //             x.unwrap().into_path(),
    //             directory,
    //         ))
    //     })
    //     .collect::<Vec<tokio::task::JoinHandle<fpm::Result<()>>>>();
    // futures::future::join_all(all_files).await;

    for x in ignore_paths.build() {
        if let Ok(file_found) = process_file(x?.into_path(), config.root.as_str()).await {
            documents.push(file_found);
        }
    }
    documents.sort_by_key(|v| v.get_id());

    Ok(documents)
}

pub fn package_ignores() -> Result<ignore::overrides::Override, ignore::Error> {
    let mut overrides = ignore::overrides::OverrideBuilder::new("./");
    overrides.add("!.history")?;
    overrides.add("!.packages")?;
    overrides.add("!.tracks")?;
    overrides.add("!FPM")?;
    overrides.add("!rust-toolchain")?;
    overrides.add("!.build")?;
    overrides.build()
}

pub(crate) async fn process_file(doc_path: std::path::PathBuf, dir: &str) -> fpm::Result<File> {
    if !&doc_path.is_dir() {
        let doc_path_str = doc_path.to_str().unwrap();
        if let Some((_, id)) = std::fs::canonicalize(&doc_path)?
            .to_str()
            .unwrap()
            .rsplit_once(format!("{}/", dir).as_str())
        {
            return Ok(match id.rsplit_once(".") {
                Some((_, "ftd")) => File::Ftd(Document {
                    id: id.to_string(),
                    content: tokio::fs::read_to_string(&doc_path).await?,
                    parent_path: dir.to_string(),
                    depth: doc_path_str.split('/').count() - 1,
                }),
                Some((doc_name, "md")) => File::Markdown(Document {
                    id: if doc_name == "README"
                        && !(std::path::Path::new("./index.ftd").exists()
                            || std::path::Path::new("./index.md").exists())
                    {
                        "index.md".to_string()
                    } else {
                        id.to_string()
                    },
                    content: tokio::fs::read_to_string(&doc_path).await?,
                    parent_path: dir.to_string(),
                    depth: doc_path_str.split('/').count() - 1,
                }),
                _ => File::Static(Static {
                    id: id.to_string(),
                    base_path: dir.to_string(),
                    depth: doc_path_str.split('/').count() - 1,
                }),
            });
        }
    }
    Err(fpm::Error::ConfigurationError {
        message: format!("{:?} should be a file", doc_path),
    })
}
