#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug, PartialEq, Eq)]
pub enum SyncStatus {
    Conflict,
    NoConflict,
    CloneEditedRemoteDeleted,
    CloneDeletedRemoteEdited,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
#[serde(tag = "action")]
pub enum SyncResponseFile {
    Add {
        path: String,
        status: SyncStatus,
        content: Vec<u8>,
    },
    Update {
        path: String,
        status: SyncStatus,
        content: Vec<u8>,
    },
    Delete {
        path: String,
        status: SyncStatus,
        content: Vec<u8>,
    },
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub struct File {
    pub path: String,
    pub content: Vec<u8>,
}

#[derive(serde::Serialize, serde::Deserialize, std::fmt::Debug)]
pub struct SyncResponse {
    pub files: Vec<SyncResponseFile>,
    pub dot_history: Vec<File>,
    pub latest_ftd: String,
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
#[serde(tag = "action")]
pub enum SyncRequestFile {
    Add { path: String, content: Vec<u8> },
    Update { path: String, content: Vec<u8> },
    Delete { path: String },
}

#[derive(serde::Deserialize, serde::Serialize, std::fmt::Debug)]
pub struct SyncRequest {
    pub package_name: String,
    pub files: Vec<SyncRequestFile>,
    pub latest_ftd: String,
}

/// Steps
/// Read latest.ftd and create snapshot version
/// Iterate over Added files, create them and update new version in latest.ftd
/// Iterate over Deleted Files, If version are same remove it from remote otherwise send updated file
/// Iterate over Update Files, get the base file according to client latest.ftd and apply three way merge,
/// If no conflict merge it, update file on remote and send back new content as Updated
/// If conflict occur, Then send back updated version in latest.ftd with conflicted content
///
pub async fn sync(
    req: &fastn_core::http::Request,
    sync_req: SyncRequest,
) -> fastn_core::Result<fastn_core::http::Response> {
    dbg!("remote server call", &sync_req.package_name);

    match sync_worker(req, sync_req).await {
        Ok(data) => fastn_core::http::api_ok(data),
        Err(err) => fastn_core::http::api_error(err.to_string()),
    }
}

pub(crate) async fn sync_worker(
    req: &fastn_core::http::Request,
    request: SyncRequest,
) -> fastn_core::Result<SyncResponse> {
    use itertools::Itertools;

    // TODO: Need to call at once only
    let config = fastn_core::Config::read(None, false, Some(req)).await?;
    let mut snapshots = fastn_core::snapshot::get_latest_snapshots(&config.root).await?;
    let client_snapshots = fastn_core::snapshot::resolve_snapshots(&request.latest_ftd).await?;
    // let latest_ftd = tokio::fs::read_to_string(config.history_dir().join(".latest.ftd")).await?;
    let timestamp = fastn_core::timestamp_nanosecond();
    let mut synced_files = std::collections::HashMap::new();
    for file in request.files.iter() {
        match file {
            SyncRequestFile::Add { path, content } => {
                // We need to check if, file is already available on server
                fastn_core::utils::update1(&config.root, path, content).await?;

                let snapshot_path =
                    fastn_core::utils::history_path(path, config.root.as_str(), &timestamp);

                if let Some((dir, _)) = snapshot_path.as_str().rsplit_once('/') {
                    tokio::fs::create_dir_all(dir).await?;
                }

                tokio::fs::copy(config.root.join(path), snapshot_path).await?;
                snapshots.insert(path.to_string(), timestamp);
                // Create a new file
                // Take snapshot
                // Update version in latest.ftd
            }
            SyncRequestFile::Delete { path } => {
                // Case: Need to handle the where client says delete but serve says modified
                // If the value of server's snapshot is greater than client snapshot
                let remote_timestamp = snapshots
                    .get(path.as_str())
                    .ok_or_else(|| fastn_core::Error::APIResponseError("".to_string()))?;
                let client_timestamp = client_snapshots
                    .get(path.as_str())
                    .ok_or_else(|| fastn_core::Error::APIResponseError("".to_string()))?;

                let snapshot_path =
                    fastn_core::utils::history_path(path, config.root.as_str(), remote_timestamp);

                let data = tokio::fs::read(snapshot_path).await?;

                // if: Client Says Deleted and server says modified
                // that means Remote timestamp is greater than client timestamp
                if remote_timestamp.gt(client_timestamp) {
                    synced_files.insert(
                        path.to_string(),
                        SyncResponseFile::Update {
                            path: path.to_string(),
                            status: SyncStatus::CloneDeletedRemoteEdited,
                            content: data,
                        },
                    );
                } else {
                    // else: both should have same version,
                    // client version(timestamp) can never be greater than server's version
                    if config.root.join(path).exists() {
                        tokio::fs::remove_file(config.root.join(path)).await?;
                    }
                    snapshots.remove(path);
                }
            }
            SyncRequestFile::Update { path, content } => {
                let client_snapshot_timestamp = client_snapshots.get(path).ok_or_else(|| {
                    fastn_core::Error::APIResponseError(format!(
                        "path should be available in latest.ftd {}",
                        path
                    ))
                })?;

                // if: Server has that file
                if let Some(snapshot_timestamp) = snapshots.get(path) {
                    // No conflict case, Only client modified the file
                    if client_snapshot_timestamp.eq(snapshot_timestamp) {
                        fastn_core::utils::update1(&config.root, path, content).await?;
                        let snapshot_path =
                            fastn_core::utils::history_path(path, config.root.as_str(), &timestamp);
                        tokio::fs::copy(config.root.join(path), snapshot_path).await?;
                        snapshots.insert(path.to_string(), timestamp);
                    } else {
                        // else: Both has modified the same file
                        // TODO: Need to handle static files like images, don't require merging
                        let ancestor_path = fastn_core::utils::history_path(
                            path,
                            config.root.as_str(),
                            client_snapshot_timestamp,
                        );
                        let ancestor_content = tokio::fs::read_to_string(ancestor_path).await?;
                        let ours_path = fastn_core::utils::history_path(
                            path,
                            config.root.as_str(),
                            snapshot_timestamp,
                        );
                        let theirs_content = tokio::fs::read_to_string(ours_path).await?;
                        let ours_content = String::from_utf8(content.clone())
                            .map_err(|e| fastn_core::Error::APIResponseError(e.to_string()))?;

                        match diffy::MergeOptions::new()
                            .set_conflict_style(diffy::ConflictStyle::Merge)
                            .merge(&ancestor_content, &ours_content, &theirs_content)
                        {
                            Ok(data) => {
                                fastn_core::utils::update1(&config.root, path, data.as_bytes())
                                    .await?;
                                let snapshot_path = fastn_core::utils::history_path(
                                    path,
                                    config.root.as_str(),
                                    &timestamp,
                                );
                                tokio::fs::copy(config.root.join(path), snapshot_path).await?;
                                snapshots.insert(path.to_string(), timestamp);
                                synced_files.insert(
                                    path.to_string(),
                                    SyncResponseFile::Update {
                                        path: path.to_string(),
                                        status: SyncStatus::NoConflict,
                                        content: data.as_bytes().to_vec(),
                                    },
                                );
                            }
                            Err(data) => {
                                // Return conflicted content
                                synced_files.insert(
                                    path.to_string(),
                                    SyncResponseFile::Update {
                                        path: path.to_string(),
                                        status: SyncStatus::Conflict,
                                        content: data.as_bytes().to_vec(),
                                    },
                                );
                            }
                        }
                    }
                } else {
                    // else: Server don't have that file
                    // If client says edited and server says deleted
                    // That means at server side there will not any entry in latest.ftd
                    synced_files.insert(
                        path.to_string(),
                        SyncResponseFile::Update {
                            path: path.to_string(),
                            status: SyncStatus::CloneEditedRemoteDeleted,
                            content: content.clone(),
                        },
                    );
                }
            }
        }
    }

    client_current_files(&config, &snapshots, &client_snapshots, &mut synced_files).await?;

    let history_files = clone_history_files(&config, &snapshots, &client_snapshots).await?;

    fastn_core::snapshot::create_latest_snapshots(
        &config,
        &snapshots
            .into_iter()
            .map(|(filename, timestamp)| fastn_core::Snapshot {
                filename,
                timestamp,
            })
            .collect_vec(),
    )
    .await?;

    let latest_ftd = tokio::fs::read_to_string(config.latest_ftd()).await?;

    let r = SyncResponse {
        files: synced_files.into_values().collect_vec(),
        dot_history: history_files,
        latest_ftd,
    };
    Ok(r)
}

fn snapshot_diff(
    server_snapshot: &std::collections::BTreeMap<String, u128>,
    client_snapshot: &std::collections::BTreeMap<String, u128>,
) -> std::collections::BTreeMap<String, u128> {
    let mut diff = std::collections::BTreeMap::new();
    for (snapshot_path, timestamp) in server_snapshot {
        match client_snapshot.get(snapshot_path) {
            Some(client_timestamp) if client_timestamp.lt(timestamp) => {
                diff.insert(snapshot_path.to_string(), *timestamp);
            }
            None => {
                diff.insert(snapshot_path.to_string(), *timestamp);
            }
            _ => {}
        };
    }
    diff
}

/// Send back Updated files(Current Directory)
///
/// Find all newly added files which are not in client latest.ftd
/// Find all the Update files at server, need to find out different snapshots in latest.ftd
/// Find deleted files, entry will not available in server's latest.ftd but will be available
/// client's latest.ftd
///
/// Send back all new .history files
///
/// find difference between client's latest.ftd and server's latest.ftd and send back those files
///
/// Send latest.ftd file as well

async fn client_current_files(
    config: &fastn_core::Config,
    server_snapshot: &std::collections::BTreeMap<String, u128>,
    client_snapshot: &std::collections::BTreeMap<String, u128>,
    synced_files: &mut std::collections::HashMap<String, SyncResponseFile>,
) -> fastn_core::Result<()> {
    // Newly Added and Updated files
    let diff = snapshot_diff(server_snapshot, client_snapshot);
    for (path, _) in diff.iter() {
        if !synced_files.contains_key(path) {
            let content = tokio::fs::read(config.root.join(path)).await?;
            synced_files.insert(
                path.clone(),
                SyncResponseFile::Add {
                    path: path.clone(),
                    status: SyncStatus::NoConflict,
                    content,
                },
            );
        }
    }

    // Deleted files

    let diff = client_snapshot
        .iter()
        .filter(|(path, _)| !server_snapshot.contains_key(path.as_str()));

    // If already in synced files need to handle that case
    for (path, _) in diff {
        if !synced_files.contains_key(path) {
            synced_files.insert(
                path.clone(),
                SyncResponseFile::Delete {
                    path: path.clone(),
                    status: SyncStatus::NoConflict,
                    content: vec![],
                },
            );
        }
    }

    Ok(())
}

async fn clone_history_files(
    config: &fastn_core::Config,
    server_snapshot: &std::collections::BTreeMap<String, u128>,
    client_snapshot: &std::collections::BTreeMap<String, u128>,
) -> fastn_core::Result<Vec<File>> {
    use itertools::Itertools;

    let diff = snapshot_diff(server_snapshot, client_snapshot);

    let history = ignore::WalkBuilder::new(config.history_dir())
        .build()
        .into_iter()
        .flatten()
        .map(|x| {
            x.into_path()
                .to_str()
                .unwrap()
                .trim_start_matches(config.history_dir().as_str())
                .trim_matches('/')
                .to_string()
        })
        .collect::<Vec<String>>();

    let mut dot_history = vec![];
    for (path, _) in diff.iter() {
        let client_timestamp = client_snapshot.get(path);
        let history_paths = get_all_timestamps(path, history.as_slice())?
            .into_iter()
            .filter(|x| client_timestamp.map(|c| x.0.gt(c)).unwrap_or(true))
            .collect_vec();
        for (_, path) in history_paths {
            let content = tokio::fs::read(config.history_dir().join(&path)).await?;
            dot_history.push(File { path, content });
        }
    }
    Ok(dot_history)
}

fn get_all_timestamps(path: &str, history: &[String]) -> fastn_core::Result<Vec<(u128, String)>> {
    let (path_prefix, ext) = if let Some((path_prefix, ext)) = path.rsplit_once('.') {
        (format!("{}.", path_prefix), Some(ext))
    } else {
        (format!("{}.", path), None)
    };
    let mut timestamps = vec![];
    for path in history.iter().filter_map(|p| p.strip_prefix(&path_prefix)) {
        let (timestamp, extension) = if let Some((timestamp, extension)) = path.rsplit_once('.') {
            (timestamp, Some(extension))
        } else {
            (path, None)
        };
        let timestamp = timestamp.parse::<u128>().unwrap();
        if ext.eq(&extension) {
            timestamps.push((timestamp, format!("{}{}", path_prefix, path)));
        }
    }
    Ok(timestamps)
}

// #[derive(Debug, std::fmt::Display)]
// struct ApiResponseError {
//     message: String,
//     api_ok: bool,
// }
