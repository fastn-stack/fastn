use itertools::Itertools;
use sha2::Digest;

pub async fn sync2(config: &fpm::Config, files: Option<Vec<String>>) -> fpm::Result<()> {
    let mut workspace = config.read_workspace().await?;
    if let Some(ref files) = files {
        workspace = workspace
            .into_iter()
            .filter(|v| files.contains(&v.filename))
            .collect_vec();
    }
    let changed_files = get_changed_files(config, workspace.as_slice()).await?;
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
    Ok(())
}

async fn update_workspace(
    response: &fpm::apis::sync2::SyncResponse,
    workspace: &mut Vec<fpm::workspace::WorkspaceEntry>,
) -> fpm::Result<()> {
    let mut new_workspace: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> =
        workspace
            .into_iter()
            .map(|v| (v.filename.to_string(), v.clone()))
            .collect();
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
        new_workspace.insert(file.to_string(), file_edit.to_workspace(file));
    }
    *workspace = new_workspace.into_values().collect_vec();
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

async fn get_changed_files(
    config: &fpm::Config,
    workspace: &[fpm::workspace::WorkspaceEntry],
) -> fpm::Result<Vec<fpm::apis::sync2::SyncRequestFile>> {
    let mut changed_files = vec![];
    for workspace_entry in workspace {
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
        let version = if let Some(version) = workspace_entry.version {
            version
        } else {
            changed_files.push(fpm::apis::sync2::SyncRequestFile::Add {
                path: workspace_entry.filename.to_string(),
                content,
            });
            continue;
        };
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
