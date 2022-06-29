use crate::apis::sync::SyncResponseFile;
use fpm::apis::sync::SyncStatus;
use fpm::Config;
use itertools::Itertools;

pub async fn sync(config: &fpm::Config, files: Option<Vec<String>>) -> fpm::Result<()> {
    // Read All the Document
    // Get all the updated, added and deleted files
    // Get Updated Files -> If content differs from latest snapshot
    // Get Added Files -> If files does not present in latest snapshot
    // Get Deleted Files -> If file present in latest.ftd and not present in directory
    // Send to fpm server

    let documents = if let Some(ref files) = files {
        let files = files
            .to_vec()
            .into_iter()
            .map(|x| config.root.join(x))
            .collect::<Vec<camino::Utf8PathBuf>>();
        fpm::paths_to_files(config.package.name.as_str(), files, config.root.as_path()).await?
    } else {
        config.get_files(&config.package).await?
    };

    tokio::fs::create_dir_all(config.history_dir()).await?;

    let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;

    let latest_ftd = tokio::fs::read_to_string(config.history_dir().join(".latest.ftd"))
        .await
        .unwrap_or_else(|_| "".to_string());

    let changed_files = get_changed_files(config, &documents, &snapshots).await?;
    let request = fpm::apis::sync::SyncRequest {
        files: changed_files,
        package_name: config.package.name.to_string(),
        latest_ftd,
    };
    let response = send_to_fpm_serve(&request).await?;
    update_current_directory(config, &response.files).await?;
    update_history(config, &response.dot_history, &response.latest_ftd).await?;
    on_conflict(config, &response, &request).await?;
    collect_garbage(config).await?;

    // Tumhe chalana hi nahi chahte hai hum, koi aur chalaye to chalaye
    if false {
        let timestamp = fpm::timestamp_nanosecond();
        let mut modified_files = vec![];
        let mut new_snapshots = vec![];
        for doc in documents {
            let (snapshot, is_modified) = write(&doc, timestamp, &snapshots).await?;
            if is_modified {
                modified_files.push(snapshot.filename.to_string());
            }
            new_snapshots.push(snapshot);
        }

        if let Some(file) = files {
            let snapshot_id = new_snapshots
                .iter()
                .map(|v| v.filename.to_string())
                .collect::<Vec<String>>();
            for (k, timestamp) in snapshots.iter() {
                if !snapshot_id.contains(k) && file.contains(k) {
                    continue;
                }
                if !snapshot_id.contains(k) {
                    new_snapshots.push(fpm::Snapshot {
                        filename: k.clone(),
                        timestamp: *timestamp,
                    })
                }
            }
        }

        for key in snapshots.keys() {
            if new_snapshots.iter().filter(|v| v.filename.eq(key)).count() == 0 {
                modified_files.push(key.clone());
            }
        }

        if modified_files.is_empty() {
            println!("Everything is upto date.");
        } else {
            fpm::snapshot::create_latest_snapshots(config, &new_snapshots).await?;
            println!(
                "Repo for {} is github, directly syncing with .history.",
                config.package.name
            );
            for file in modified_files {
                println!("{}", file);
            }
        }
    }
    Ok(())
}

async fn get_changed_files(
    config: &fpm::Config,
    files: &[fpm::File],
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fpm::Result<Vec<fpm::apis::sync::SyncRequestFile>> {
    use sha2::Digest;
    // Get all the updated, added and deleted files
    // Get Updated Files -> If content differs from latest snapshot
    // Get Added Files -> If files does not present in latest snapshot
    // Get Deleted Files -> If file present in latest.ftd and not present in files directory

    let workspace = fpm::snapshot::get_workspace(config).await?;
    let mut changed_files = Vec::new();
    for document in files.iter() {
        match workspace.get(&document.get_id()) {
            Some(workspace) if !workspace.is_resolved() => continue,
            _ => {}
        }
        if let Some(timestamp) = snapshots.get(&document.get_id()) {
            let snapshot_file_path =
                fpm::utils::history_path(&document.get_id(), &document.get_base_path(), timestamp);
            let snapshot_file_content = tokio::fs::read(&snapshot_file_path).await?;
            // Update
            let current_file_content = document.get_content();
            if sha2::Sha256::digest(&snapshot_file_content)
                .eq(&sha2::Sha256::digest(&current_file_content))
            {
                continue;
            }

            changed_files.push(fpm::apis::sync::SyncRequestFile::Update {
                path: document.get_id(),
                content: current_file_content,
            });
        } else {
            // Added
            changed_files.push(fpm::apis::sync::SyncRequestFile::Add {
                path: document.get_id(),
                content: document.get_content(),
            });
        }
    }
    let files_path = files
        .iter()
        .map(|f| f.get_id())
        .collect::<std::collections::HashSet<String>>();

    let deleted_files = snapshots
        .keys()
        .filter(|x| !files_path.contains(*x))
        .map(|f| fpm::apis::sync::SyncRequestFile::Delete {
            path: f.to_string(),
        });

    changed_files.extend(deleted_files);

    Ok(changed_files)
}

async fn write(
    doc: &fpm::File,
    timestamp: u128,
    snapshots: &std::collections::BTreeMap<String, u128>,
) -> fpm::Result<(fpm::Snapshot, bool)> {
    use sha2::Digest;
    if let Some((dir, _)) = doc.get_id().rsplit_once('/') {
        tokio::fs::create_dir_all(
            camino::Utf8PathBuf::from(doc.get_base_path())
                .join(".history")
                .join(dir),
        )
        .await?;
    }

    if let Some(timestamp) = snapshots.get(&doc.get_id()) {
        let path = fpm::utils::history_path(&doc.get_id(), &doc.get_base_path(), timestamp);
        if let Ok(current_doc) = tokio::fs::read(&doc.get_full_path()).await {
            let existing_doc = tokio::fs::read(&path).await?;

            if sha2::Sha256::digest(current_doc).eq(&sha2::Sha256::digest(existing_doc)) {
                return Ok((
                    fpm::Snapshot {
                        filename: doc.get_id(),
                        timestamp: *timestamp,
                    },
                    false,
                ));
            }
        }
    }

    let new_file_path = fpm::utils::history_path(&doc.get_id(), &doc.get_base_path(), &timestamp);

    tokio::fs::copy(doc.get_full_path(), new_file_path).await?;

    Ok((
        fpm::Snapshot {
            filename: doc.get_id(),
            timestamp,
        },
        true,
    ))
}

async fn send_to_fpm_serve(
    data: &fpm::apis::sync::SyncRequest,
) -> fpm::Result<fpm::apis::sync::SyncResponse> {
    #[derive(serde::Deserialize, std::fmt::Debug)]
    struct ApiResponse {
        message: Option<String>,
        data: Option<fpm::apis::sync::SyncResponse>,
        success: bool,
    }

    let data = serde_json::to_string(&data)?;
    let mut response = reqwest::Client::new()
        .post("http://127.0.0.1:8000/-/sync/")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(data)
        .send()?;

    let response = response.json::<ApiResponse>()?;
    if !response.success {
        return Err(fpm::Error::APIResponseError(
            response
                .message
                .unwrap_or_else(|| "Some Error occurred".to_string()),
        ));
    }

    match response.data {
        Some(data) => Ok(data),
        None => Err(fpm::Error::APIResponseError(
            "Unexpected API behaviour".to_string(),
        )),
    }
}

async fn update_current_directory(
    config: &fpm::Config,
    files: &[fpm::apis::sync::SyncResponseFile],
) -> fpm::Result<()> {
    for file in files {
        match file {
            SyncResponseFile::Add { path, content, .. } => {
                fpm::utils::update(&config.root, path, content).await?;
            }
            SyncResponseFile::Update {
                path,
                content,
                status,
            } => {
                if SyncStatus::ClientDeletedServerEdited.eq(status) {
                    println!("ClientDeletedServerEdit: {}", path);
                }
                if SyncStatus::ClientEditedServerDeleted.eq(status) {
                    println!("ClientEditedServerDeleted: {}", path);
                }
                if SyncStatus::Conflict.eq(status) {
                    println!("Conflict: {}", path);
                }
                fpm::utils::update(&config.root, path, content).await?;
            }
            SyncResponseFile::Delete { path, .. } => {
                if config.root.join(path).exists() {
                    tokio::fs::remove_file(config.root.join(path)).await?;
                }
            }
        }
    }
    Ok(())
}

async fn update_history(
    config: &fpm::Config,
    files: &[fpm::apis::sync::File],
    latest_ftd: &str,
) -> fpm::Result<()> {
    for file in files {
        fpm::utils::update(&config.history_dir(), file.path.as_str(), &file.content).await?;
    }
    fpm::utils::update(&config.history_dir(), ".latest.ftd", latest_ftd.as_bytes()).await?;
    Ok(())
}

// Steps
// create .fpm/workspace.ftd
// create .fpm/conflict directory
// If conflict occur
// - In file content will be with conflict markers
// - In conflict/<file.ftd> will contain client's content
// - In .fpm/workspace.ftd will have entry of index.ftd with data
// - name: file_name
// - base: last successful merge version from request `latest.ftd`
// - conflicted_version: from response `latest.ftd`
// - workspace: ours/theirs/conflicted

fn get_file_content<'a>(
    file_path: &'a str,
    files: &'a [fpm::apis::sync::SyncRequestFile],
) -> Option<&'a Vec<u8>> {
    for file in files {
        match file {
            fpm::apis::sync::SyncRequestFile::Add { path, content }
            | fpm::apis::sync::SyncRequestFile::Update { path, content } => {
                if file_path.eq(path) {
                    return Some(content);
                }
            }
            _ => {}
        }
    }
    None
}

async fn on_conflict(
    config: &fpm::Config,
    response: &fpm::apis::sync::SyncResponse,
    request: &fpm::apis::sync::SyncRequest,
) -> fpm::Result<()> {
    let client_snapshot = fpm::snapshot::resolve_snapshots(&request.latest_ftd).await?;
    let mut workspace = fpm::snapshot::get_workspace(config).await?;

    fn error(msg: &str) -> fpm::Error {
        fpm::Error::APIResponseError(msg.to_string())
    }

    for file in response.files.iter() {
        match file {
            SyncResponseFile::Update { path, status, .. }
            | SyncResponseFile::Add { path, status, .. }
            | SyncResponseFile::Delete { path, status, .. } => {
                if fpm::apis::sync::SyncStatus::Conflict.eq(status) {
                    let server_snapshot =
                        fpm::snapshot::resolve_snapshots(&response.latest_ftd).await?;
                    let content = get_file_content(path, request.files.as_slice())
                        .ok_or_else(|| error("File should be available in request file"))?;
                    fpm::utils::update(&config.conflicted_dir(), path, content).await?;
                    workspace.insert(
                        path.to_string(),
                        fpm::snapshot::Workspace {
                            filename: path.to_string(),
                            base: *client_snapshot
                                .get(path)
                                .ok_or_else(|| error("File should be available in request file"))?,
                            conflicted: *server_snapshot
                                .get(path)
                                .ok_or_else(|| error("File should be available in request file"))?,
                            workspace: fpm::snapshot::WorkspaceType::Conflicted,
                        },
                    );
                } else if fpm::apis::sync::SyncStatus::ClientEditedServerDeleted.eq(status) {
                    let content = get_file_content(path, request.files.as_slice())
                        .ok_or_else(|| error("File should be available in request file"))?;
                    fpm::utils::update(&config.conflicted_dir(), path, content).await?;
                    workspace.insert(
                        path.to_string(),
                        fpm::snapshot::Workspace {
                            filename: path.to_string(),
                            base: *client_snapshot
                                .get(path)
                                .ok_or_else(|| error("File should be available in request file"))?,
                            conflicted: *client_snapshot
                                .get(path)
                                .ok_or_else(|| error("File should be available in request file"))?,
                            workspace: fpm::snapshot::WorkspaceType::ClientEditedServerDeleted,
                        },
                    );
                } else if fpm::apis::sync::SyncStatus::ClientDeletedServerEdited.eq(status) {
                    let server_snapshot =
                        fpm::snapshot::resolve_snapshots(&response.latest_ftd).await?;
                    workspace.insert(
                        path.to_string(),
                        fpm::snapshot::Workspace {
                            filename: path.to_string(),
                            base: *client_snapshot
                                .get(path)
                                .ok_or_else(|| error("File should be available in request file"))?,
                            conflicted: *server_snapshot
                                .get(path)
                                .ok_or_else(|| error("File should be available in request file"))?,
                            workspace: fpm::snapshot::WorkspaceType::ClientDeletedServerEdited,
                        },
                    );
                }
            }
        }
    }

    fpm::snapshot::create_workspace(config, workspace.into_values().collect_vec().as_slice())
        .await?;

    Ok(())
}

async fn collect_garbage(config: &Config) -> fpm::Result<()> {
    let mut workspaces = fpm::snapshot::get_workspace(config).await?;

    let paths = workspaces
        .iter()
        .filter(|(_, workspace)| workspace.is_resolved())
        .map(|(path, _)| path.to_string())
        .collect_vec();

    for path in paths {
        tokio::fs::remove_file(config.conflicted_dir().join(&path)).await?;
        workspaces.remove(&path);
    }

    fpm::snapshot::create_workspace(config, workspaces.into_values().collect_vec().as_slice())
        .await?;
    Ok(())
}
