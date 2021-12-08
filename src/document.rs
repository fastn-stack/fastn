#[derive(Debug)]
pub enum FileFound {
    FTDDocument(Document),
    StaticAsset(StaticAsset),
}

impl FileFound {
    pub fn get_id(&self) -> String {
        match self {
            Self::FTDDocument(a) => a.id.as_str().to_string(),
            Self::StaticAsset(a) => a.id.as_str().to_string(),
        }
    }
}

#[derive(Debug)]
pub struct Document {
    pub id: String,
    pub document: String,
    pub base_path: String,
    pub depth: usize,
}

#[derive(Debug)]
pub struct StaticAsset {
    pub id: String,
    pub base_path: String,
    pub depth: usize,
}

pub(crate) async fn process_dir(
    directory: &str,
    config: &fpm::Config,
) -> fpm::Result<Vec<FileFound>> {
    let mut documents: Vec<FileFound> = vec![];
    let mut ignore_paths = ignore::WalkBuilder::new("./");
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
        process_file_(&mut documents, x.unwrap().into_path(), directory).await?;
    }
    documents.sort_by_key(|v| v.get_id().clone());

    return Ok(documents);

    async fn process_file_(
        documents: &mut Vec<FileFound>,
        doc_path: std::path::PathBuf,
        dir: &str,
    ) -> fpm::Result<()> {
        if !&doc_path.is_dir() {
            let doc_path_str = doc_path.to_str().unwrap();
            if let Some((_, id)) = std::fs::canonicalize(&doc_path)?
                .to_str()
                .unwrap()
                .rsplit_once(format!("{}/", dir).as_str())
            {
                documents.push(match &id.ends_with(".ftd") {
                    true => FileFound::FTDDocument(Document {
                        id: id.to_string(),
                        document: tokio::fs::read_to_string(&doc_path).await?,
                        base_path: dir.to_string(),
                        depth: doc_path_str.split('/').count() - 1,
                    }),
                    false => FileFound::StaticAsset(StaticAsset {
                        id: id.to_string(),
                        base_path: dir.to_string(),
                        depth: doc_path_str.split('/').count() - 1,
                    }),
                });
            }
        }
        Ok(())
    }
}
