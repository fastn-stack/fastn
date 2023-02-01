#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct CRMeta {
    pub title: String, // relative file name with respect to package root
    #[serde(rename = "cr-number")]
    pub cr_number: usize,
    pub open: bool,
}

impl CRMeta {
    pub(crate) fn unset_open(self) -> CRMeta {
        CRMeta {
            title: self.title,
            cr_number: self.cr_number,
            open: false,
        }
    }
}

pub(crate) async fn get_cr_meta(
    config: &fastn::Config,
    cr_number: usize,
) -> fastn::Result<fastn::cr::CRMeta> {
    let cr_meta_path = config.cr_meta_path(cr_number);
    if !cr_meta_path.exists() {
        return fastn::usage_error(format!("CR#{} doesn't exist", cr_number));
    }

    let doc = tokio::fs::read_to_string(&cr_meta_path).await?;
    resolve_cr_meta(&doc, cr_number).await
}

pub(crate) async fn resolve_cr_meta(
    content: &str,
    cr_number: usize,
) -> fastn::Result<fastn::cr::CRMeta> {
    #[derive(serde::Deserialize)]
    struct CRMetaTemp {
        pub title: String,
        pub open: Option<bool>,
    }

    impl CRMetaTemp {
        fn into_cr_meta(self, cr_number: usize) -> CRMeta {
            CRMeta {
                title: self.title,
                cr_number,
                open: self.open.unwrap_or(true),
            }
        }
    }

    if content.trim().is_empty() {
        return Err(fastn::Error::UsageError {
            message: "Content is empty in cr about".to_string(),
        });
    }
    let lib = fastn::FastnLibrary::default();
    let b = match fastn::doc::parse_ftd(".about.ftd", content, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse .about.ftd for CR#{}: {:?}", cr_number, &e);
            todo!();
        }
    };

    Ok(b.get::<CRMetaTemp>("fastn#cr-meta")?
        .into_cr_meta(cr_number))
}

pub(crate) async fn create_cr_about(
    config: &fastn::Config,
    cr_meta: &fastn::cr::CRMeta,
) -> fastn::Result<()> {
    let default_cr_about_content = fastn::package_info_about(config)?;
    fastn::utils::update(
        &config.cr_about_path(cr_meta.cr_number),
        default_cr_about_content.as_bytes(),
    )
    .await?;
    Ok(())
}

pub(crate) async fn create_cr_meta(
    config: &fastn::Config,
    cr_meta: &fastn::cr::CRMeta,
) -> fastn::Result<()> {
    let meta_content = generate_cr_meta_content(cr_meta);
    fastn::utils::update(
        &config.cr_meta_path(cr_meta.cr_number),
        meta_content.as_bytes(),
    )
    .await?;
    Ok(())
}

pub(crate) fn generate_cr_meta_content(cr_meta: &fastn::cr::CRMeta) -> String {
    let mut meta_content = format!("-- import: fastn\n\n\n-- fastn.cr-meta: {}", cr_meta.title,);
    if !cr_meta.open {
        meta_content = format!("{}\n{}", meta_content, cr_meta.open);
    }
    format!("{meta_content}\n")
}

pub(crate) async fn is_open_cr_exists(
    config: &fastn::Config,
    cr_number: usize,
) -> fastn::Result<bool> {
    get_cr_meta(config, cr_number).await.map(|v| v.open)
}

#[derive(serde::Serialize, serde::Deserialize, Debug, Default)]
pub struct CRDeleted {
    pub filename: String,
    pub version: i32,
}

impl CRDeleted {
    pub(crate) fn new(filename: &str, version: i32) -> CRDeleted {
        CRDeleted {
            filename: filename.to_string(),
            version,
        }
    }

    pub(crate) fn into_file_status(self) -> fastn::sync_utils::FileStatus {
        fastn::sync_utils::FileStatus::Delete {
            path: self.filename,
            version: self.version,
            status: fastn::sync_utils::Status::NoConflict,
        }
    }
}

pub(crate) async fn get_deleted_files(
    config: &fastn::Config,
    cr_number: usize,
) -> fastn::Result<Vec<CRDeleted>> {
    if !config.cr_path(cr_number).exists() {
        return fastn::usage_error(format!("CR#{} doesn't exist", cr_number));
    }
    let deleted_files_path = config.cr_deleted_file_path(cr_number);
    if !deleted_files_path.exists() {
        return Ok(vec![]);
    }
    let deleted_files_content = tokio::fs::read_to_string(&deleted_files_path).await?;
    resolve_cr_deleted(deleted_files_content.as_str(), cr_number).await
}

pub(crate) async fn resolve_cr_deleted(
    content: &str,
    cr_number: usize,
) -> fastn::Result<Vec<CRDeleted>> {
    if content.trim().is_empty() {
        return Ok(vec![]);
    }
    let lib = fastn::FastnLibrary::default();
    let b = match fastn::doc::parse_ftd("deleted.ftd", content, &lib) {
        Ok(v) => v,
        Err(e) => {
            eprintln!("failed to parse deleted.ftd for CR#{}: {:?}", cr_number, &e);
            todo!();
        }
    };

    Ok(b.get("fastn#cr-deleted")?)
}

pub(crate) async fn create_deleted_files(
    config: &fastn::Config,
    cr_number: usize,
    cr_deleted: &[CRDeleted],
) -> fastn::Result<()> {
    let cr_deleted_content = generate_deleted_files_content(cr_deleted);
    fastn::utils::update(
        &config.cr_deleted_file_path(cr_number),
        cr_deleted_content.as_bytes(),
    )
    .await?;
    Ok(())
}

pub(crate) fn generate_deleted_files_content(cr_deleted_files: &[CRDeleted]) -> String {
    let mut deleted_files_content = vec!["-- import: fastn".to_string()];

    for cr_deleted_file in cr_deleted_files {
        let content = format!(
            "-- fastn.cr-deleted: {}\nversion: {}",
            cr_deleted_file.filename, cr_deleted_file.version
        );
        deleted_files_content.push(content)
    }

    let content = deleted_files_content.join("\n\n");
    format!("{content}\n")
}

impl fastn::Config {
    #[allow(dead_code)]
    pub(crate) async fn get_cr_tracking_info(
        &self,
        cr_number: usize,
    ) -> fastn::Result<Vec<fastn::track::TrackingInfo>> {
        let cr_track_paths = ignore::WalkBuilder::new(self.cr_track_dir(cr_number))
            .build()
            .into_iter()
            .flatten()
            .map(|x| camino::Utf8PathBuf::from_path_buf(x.into_path()).unwrap())
            .filter(|x| x.is_file() && x.extension().map(|v| v.eq("track")).unwrap_or(false))
            .collect::<Vec<camino::Utf8PathBuf>>();
        let mut tracking_info_list = vec![];

        for cr_track_path in cr_track_paths {
            let tracked_file = cr_track_path.strip_prefix(self.track_dir())?;
            let tracked_file_str =
                fastn::cr::cr_path_to_file_name(cr_number, tracked_file.as_str())?;
            if let Some(info) = fastn::track::get_tracking_info_(&cr_track_path)
                .await?
                .into_iter()
                .find(|v| tracked_file_str.eq(&v.filename))
            {
                tracking_info_list.push(info);
            }
        }
        Ok(tracking_info_list)
    }
}

pub(crate) fn cr_path(cr_number: usize) -> String {
    format!("-/{}", cr_number)
}

pub(crate) fn cr_path_to_file_name(cr_number: usize, cr_file_path: &str) -> fastn::Result<String> {
    let cr_path = cr_path(cr_number);
    Ok(cr_file_path
        .trim_matches('/')
        .trim_start_matches(cr_path.as_str())
        .to_string())
}

pub(crate) fn get_cr_path_from_url(path: &str) -> Option<usize> {
    let mut cr_number = None;
    if let Some(path) = path.strip_prefix("-/") {
        if let Some((cr, _)) = path.split_once('/') {
            if let Ok(cr) = cr.parse::<usize>() {
                cr_number = Some(cr);
            }
        }
    }
    cr_number
}

pub(crate) fn get_id_from_cr_id(id: &str, cr_number: usize) -> fastn::Result<String> {
    let cr_path = cr_path(cr_number);
    if let Some(id) = id.trim_start_matches('/').strip_prefix(cr_path.as_str()) {
        return Ok(if !id.ends_with('/') {
            format!("{}/", id)
        } else {
            id.to_string()
        });
    }
    fastn::usage_error(format!("`{}` is not a cr id", id))
}

#[derive(Debug)]
pub(crate) struct FileInfo {
    pub path: String,
    pub content: Vec<u8>,
}

pub(crate) async fn cr_clone_file_info(
    config: &fastn::Config,
    cr_number: usize,
) -> fastn::Result<std::collections::HashMap<String, FileInfo>> {
    use itertools::Itertools;

    let mut file_info = std::collections::HashMap::new();

    let remote_manifest: std::collections::BTreeMap<String, fastn::history::FileEdit> = config
        .get_remote_manifest(true)
        .await?
        .into_iter()
        .filter(|(k, _)| !(k.starts_with("-/") || k.starts_with(".tracks/")))
        .collect();

    for (filename, file_edit) in remote_manifest {
        if file_edit.is_deleted() {
            continue;
        }
        let file_path = config.history_path(filename.as_str(), file_edit.version);
        let content = tokio::fs::read(&file_path).await?;

        let path = config.path_without_root(&file_path)?;

        file_info.insert(
            filename,
            FileInfo {
                path: path.to_string(),
                content,
            },
        );
    }

    let cr_path = fastn::cr::cr_path(cr_number);

    let cr_workspace = config
        .get_clone_workspace()
        .await?
        .into_iter()
        .filter_map(|(k, v)| {
            k.strip_prefix(cr_path.as_str())
                .map(|path| (path.trim_start_matches('/').to_string(), v))
        })
        .collect::<std::collections::HashMap<String, fastn::workspace::WorkspaceEntry>>();

    let deleted_files_path = config.cr_deleted_file_path(cr_number);
    let deleted_file_str = config.path_without_root(&deleted_files_path)?;
    for (filename, workspace_entry) in cr_workspace {
        if workspace_entry.deleted.unwrap_or(false) {
            continue;
        }
        if workspace_entry.filename.eq(&deleted_file_str) {
            let cr_deleted_path = if let Some(version) = workspace_entry.version {
                config.history_path(workspace_entry.filename.as_str(), version)
            } else {
                config.root.join(workspace_entry.filename)
            };
            let cr_deleted_files = tokio::fs::read_to_string(cr_deleted_path).await?;
            fastn::cr::resolve_cr_deleted(cr_deleted_files.as_str(), cr_number)
                .await?
                .into_iter()
                .map(|v| {
                    file_info.remove(v.filename.as_str());
                })
                .collect_vec();
            continue;
        }
        let content = tokio::fs::read(config.root.join(workspace_entry.filename.as_str())).await?;

        file_info.insert(
            filename,
            FileInfo {
                path: workspace_entry.filename.to_string(),
                content,
            },
        );
    }

    Ok(file_info)
}

#[allow(dead_code)]
pub(crate) async fn cr_remote_file_info(
    config: &fastn::Config,
    cr_number: usize,
) -> fastn::Result<std::collections::HashMap<String, FileInfo>> {
    use itertools::Itertools;

    let mut file_info = std::collections::HashMap::new();

    let remote_manifest: std::collections::HashMap<String, fastn::history::FileEdit> = config
        .get_remote_manifest(true)
        .await?
        .into_iter()
        .filter(|(k, _)| !(k.starts_with("-/") || k.starts_with(".tracks/")))
        .collect();

    for (filename, file_edit) in remote_manifest {
        if file_edit.is_deleted() {
            continue;
        }
        let file_path = config.history_path(filename.as_str(), file_edit.version);
        let content = tokio::fs::read(&file_path).await?;

        let path = config.path_without_root(&file_path)?;

        file_info.insert(
            filename,
            FileInfo {
                path: path.to_string(),
                content,
            },
        );
    }

    let cr_manifest: std::collections::HashMap<String, fastn::history::FileEdit> = config
        .get_cr_manifest(cr_number)
        .await?
        .into_iter()
        .filter(|(k, _)| !k.starts_with(".tracks/"))
        .collect();

    let deleted_files_path = config.cr_deleted_file_path(cr_number);
    let deleted_file_str = config.path_without_root(&deleted_files_path)?;
    let cr_path = fastn::cr::cr_path(cr_number);
    for (filename, file_edit) in cr_manifest {
        if file_edit.is_deleted() {
            continue;
        }

        if filename.eq(&deleted_file_str) {
            let cr_deleted_path = config.history_path(filename.as_str(), file_edit.version);
            let cr_deleted_files = tokio::fs::read_to_string(cr_deleted_path).await?;
            fastn::cr::resolve_cr_deleted(cr_deleted_files.as_str(), cr_number)
                .await?
                .into_iter()
                .map(|v| {
                    file_info.remove(v.filename.as_str());
                })
                .collect_vec();
            continue;
        }

        let file_path = config.history_path(filename.as_str(), file_edit.version);
        let content = tokio::fs::read(&file_path).await?;

        let path = config.path_without_root(&file_path)?;

        file_info.insert(
            filename.trim_start_matches(cr_path.as_str()).to_string(),
            FileInfo {
                path: path.to_string(),
                content,
            },
        );
    }

    Ok(file_info)
}
