use itertools::Itertools;
use sha2::Digest;

pub async fn sync2(config: &fpm::Config, files: Option<Vec<String>>) -> fpm::Result<()> {
    let mut file_list = config.read_workspace().await?;
    let mut workspace: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> =
        file_list
            .iter()
            .map(|v| (v.filename.to_string(), v.clone()))
            .collect();
    if let Some(ref files) = files {
        file_list = file_list
            .into_iter()
            .filter(|v| files.contains(&v.filename))
            .collect_vec();
    }
    let changed_files = get_changed_files(config, file_list.as_slice(), &mut workspace).await?;
    let history = tokio::fs::read_to_string(config.history_file()).await?;
    let sync_request = fpm::apis::sync2::SyncRequest {
        package_name: config.package.name.to_string(),
        files: changed_files,
        history,
    };
    let response = send_to_fpm_serve(&sync_request).await?;
    update_current_directory(config, &response.files).await?;
    update_history(config, &response.dot_history, &response.latest_ftd).await?;
    update_workspace(&response, &mut workspace).await?;
    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;
    Ok(())
}

async fn update_workspace(
    response: &fpm::apis::sync2::SyncResponse,
    workspace: &mut std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
) -> fpm::Result<()> {
    let server_history = fpm::history::FileHistory::from_ftd(response.latest_ftd.as_str())?;
    let server_latest =
        fpm::history::FileHistory::get_latest_file_edits(server_history.as_slice())?;
    let conflicted_files = response
        .files
        .iter()
        .filter_map(|v| {
            if v.is_conflicted() {
                Some(v.path())
            } else {
                None
            }
        })
        .collect_vec();
    for (file, file_edit) in server_latest.iter() {
        if conflicted_files.contains(file) {
            continue;
        }
        workspace.insert(file.to_string(), file_edit.to_workspace(file));
    }
    for deleted_files in response.files.iter().filter_map(|v| {
        if !v.is_conflicted() && v.is_deleted() {
            Some(v.path())
        } else {
            None
        }
    }) {
        workspace.remove(&deleted_files);
    }
    Ok(())
}

async fn update_history(
    config: &fpm::Config,
    files: &[fpm::apis::sync2::File],
    latest_ftd: &str,
) -> fpm::Result<()> {
    for file in files {
        fpm::utils::update(
            &config.server_history_dir().join(file.path.as_str()),
            &file.content,
        )
        .await?;
    }
    fpm::utils::update(&config.history_file(), latest_ftd.as_bytes()).await?;
    Ok(())
}

async fn update_current_directory(
    config: &fpm::Config,
    files: &[fpm::apis::sync2::SyncResponseFile],
) -> fpm::Result<()> {
    for file in files {
        match file {
            fpm::apis::sync2::SyncResponseFile::Add { path, content, .. } => {
                fpm::utils::update(&config.root.join(path), content).await?;
            }
            fpm::apis::sync2::SyncResponseFile::Update {
                path,
                content,
                status,
            } => {
                if fpm::apis::sync2::SyncStatus::ClientDeletedServerEdited.eq(status) {
                    println!("ClientDeletedServerEdit: {}", path);
                } else if fpm::apis::sync2::SyncStatus::ClientEditedServerDeleted.eq(status) {
                    println!("ClientEditedServerDeleted: {}", path);
                } else if fpm::apis::sync2::SyncStatus::Conflict.eq(status) {
                    println!("Conflict: {}", path);
                } else {
                    fpm::utils::update(&config.root.join(path), content).await?;
                }
            }
            fpm::apis::sync2::SyncResponseFile::Delete { path, .. } => {
                if config.root.join(path).exists() {
                    tokio::fs::remove_file(config.root.join(path)).await?;
                }
            }
        }
    }
    Ok(())
}

async fn get_changed_files_wrt_server_latest(
    config: &fpm::Config,
    files: &mut Vec<fpm::apis::sync2::SyncRequestFile>,
    workspace: &mut std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
) -> fpm::Result<()> {
    let mut remove_files = vec![];
    let server_latest = config.get_latest_file_edits_with_deleted().await?;
    for (index, file) in files.iter_mut().enumerate() {
        match file {
            fpm::apis::sync2::SyncRequestFile::Add { path, content } => {
                let server_version = if let Some(file_edit) = server_latest.get(path) {
                    if file_edit.is_deleted() {
                        continue;
                    }
                    file_edit.version
                } else {
                    continue;
                };
                let history_path = config.history_path(path, server_version);
                let history_content = tokio::fs::read(history_path).await?;
                if sha2::Sha256::digest(content).eq(&sha2::Sha256::digest(history_content)) {
                    workspace.insert(
                        path.to_string(),
                        fpm::workspace::WorkspaceEntry {
                            filename: path.to_string(),
                            deleted: None,
                            version: Some(server_version),
                        },
                    );
                }

                remove_files.push(index);
            }
            fpm::apis::sync2::SyncRequestFile::Update {
                path,
                content,
                version,
            } => {
                let server_file_edit = if let Some(file_edit) = server_latest.get(path) {
                    file_edit
                } else {
                    continue;
                };

                if server_file_edit.is_deleted() {
                    // Conflict: ClientEditedServerDeleted
                    remove_files.push(index);
                    continue;
                }

                if server_file_edit.version.eq(version) {
                    continue;
                }

                let ancestor_content = if let Ok(content) =
                    tokio::fs::read_to_string(config.history_path(path, *version)).await
                {
                    content
                } else {
                    // binary file like images, can't resolve conflict
                    remove_files.push(index);
                    continue;
                };

                // attempt resolving conflict
                let theirs_content =
                    tokio::fs::read_to_string(config.history_path(path, server_file_edit.version))
                        .await?;
                let ours_content = String::from_utf8(content.clone())?;

                match diffy::MergeOptions::new()
                    .set_conflict_style(diffy::ConflictStyle::Merge)
                    .merge(&ancestor_content, &ours_content, &theirs_content)
                {
                    Ok(data) => {
                        tokio::fs::write(path, &data).await?;
                        *content = data.as_bytes().to_vec();
                        *version = server_file_edit.version;
                    }
                    Err(_) => {
                        // can't resolve conflict, so cannot sync
                        remove_files.push(index);
                    }
                }
            }
            fpm::apis::sync2::SyncRequestFile::Delete { path, version } => {
                let server_file_edit = if let Some(server_file_edit) = server_latest.get(path) {
                    server_file_edit
                } else {
                    remove_files.push(index);
                    workspace.remove(path);
                    continue;
                };
                if server_file_edit.is_deleted() {
                    remove_files.push(index);
                    workspace.remove(path);
                    continue;
                }
                if !server_file_edit.version.eq(version) {
                    // Conflict modified by server and deleted by client
                    remove_files.push(index);
                }
            }
        }
    }
    *files = files
        .iter_mut()
        .enumerate()
        .filter_map(|(k, v)| {
            if !remove_files.contains(&k) {
                Some(v.to_owned())
            } else {
                None
            }
        })
        .collect_vec();
    Ok(())
}

async fn get_changed_files_wrt_workspace(
    config: &fpm::Config,
    files: &[fpm::workspace::WorkspaceEntry],
) -> fpm::Result<Vec<fpm::apis::sync2::SyncRequestFile>> {
    let mut changed_files = vec![];
    for workspace_entry in files {
        let version = if let Some(version) = workspace_entry.version {
            version
        } else {
            let content =
                tokio::fs::read(config.root.join(workspace_entry.filename.as_str())).await?;
            changed_files.push(fpm::apis::sync2::SyncRequestFile::Add {
                path: workspace_entry.filename.to_string(),
                content,
            });
            continue;
        };
        if workspace_entry.deleted.unwrap_or(false) {
            changed_files.push(fpm::apis::sync2::SyncRequestFile::Delete {
                path: workspace_entry.filename.to_string(),
                version: workspace_entry.version.ok_or(fpm::Error::UsageError {
                    message: format!(
                        "{}, which is to be deleted, doesn't define version in workspace",
                        workspace_entry.filename
                    ),
                })?,
            });
            continue;
        }

        let content = tokio::fs::read(config.root.join(workspace_entry.filename.as_str())).await?;
        let history_path = config.history_path(workspace_entry.filename.as_str(), version);
        let history_content = tokio::fs::read(history_path).await?;
        if sha2::Sha256::digest(&content).eq(&sha2::Sha256::digest(&history_content)) {
            continue;
        }
        changed_files.push(fpm::apis::sync2::SyncRequestFile::Update {
            path: workspace_entry.filename.to_string(),
            content,
            version,
        });
    }
    Ok(changed_files)
}

async fn get_changed_files(
    config: &fpm::Config,
    files: &[fpm::workspace::WorkspaceEntry],
    workspace: &mut std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry>,
) -> fpm::Result<Vec<fpm::apis::sync2::SyncRequestFile>> {
    let mut changed_files = get_changed_files_wrt_workspace(config, files).await?;
    get_changed_files_wrt_server_latest(config, &mut changed_files, workspace).await?;
    Ok(changed_files)
}

async fn send_to_fpm_serve(
    data: &fpm::apis::sync2::SyncRequest,
) -> fpm::Result<fpm::apis::sync2::SyncResponse> {
    #[derive(serde::Deserialize, std::fmt::Debug)]
    struct ApiResponse {
        message: Option<String>,
        data: Option<fpm::apis::sync2::SyncResponse>,
        success: bool,
    }

    let data = serde_json::to_string(&data)?;
    let mut response = reqwest::Client::new()
        .post("http://127.0.0.1:8000/-/sync2/")
        .header(reqwest::header::CONTENT_TYPE, "application/json")
        .body(data)
        .send()?;
    dbg!("send_to_fpm_serve", &response.status());

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
