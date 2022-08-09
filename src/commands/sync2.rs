use itertools::Itertools;

pub async fn sync2(config: &fpm::Config, files: Option<Vec<String>>) -> fpm::Result<()> {
    let file_list = config.read_workspace().await?;
    let mut workspace: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> =
        file_list
            .iter()
            .map(|v| (v.filename.to_string(), v.clone()))
            .collect();
    let changed_files = {
        let mut changed_files = config
            .get_files_status_with_workspace(&mut workspace)
            .await?;
        if let Some(ref files) = files {
            changed_files = changed_files
                .into_iter()
                .filter(|v| files.contains(&v.get_file_path()))
                .collect();
        }
        changed_files
            .into_iter()
            .filter_map(|v| v.sync_request())
            .collect_vec()
    };
    let history = tokio::fs::read_to_string(config.history_file()).await?;
    let sync_request = fpm::apis::sync2::SyncRequest {
        package_name: config.package.name.to_string(),
        files: changed_files,
        history,
    };
    let response = send_to_fpm_serve(&sync_request).await?;
    update_current_directory(config, &response).await?;
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
    for (file, file_edit) in server_latest.into_iter() {
        if conflicted_files.contains(&file) || file_edit.is_deleted() {
            continue;
        }
        workspace.insert(file.to_string(), file_edit.into_workspace(&file));
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
    response: &fpm::apis::sync2::SyncResponse,
) -> fpm::Result<()> {
    for file in response.files.iter() {
        match file {
            fpm::apis::sync2::SyncResponseFile::Add {
                path,
                content,
                status,
            } => {
                if status.add_add_conflict() {
                    println!("CloneAddedRemoteAdded: {}", path);
                } else {
                    fpm::utils::update(&config.root.join(path), content).await?;
                }
            }
            fpm::apis::sync2::SyncResponseFile::Update {
                path,
                content,
                status,
            } => {
                if status.edit_delete_conflict() {
                    println!("CloneDeletedRemoteEdit: {}", path);
                } else if status.delete_edit_conflict() {
                    println!("CloneEditedRemoteDeleted: {}", path);
                } else if status.edit_edit_conflict() {
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
