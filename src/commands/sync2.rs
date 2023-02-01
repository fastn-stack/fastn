pub async fn sync2(
    config: &fastn::Config,
    files: Option<Vec<String>>,
    // cr_number: Option<&str>,
) -> fastn::Result<()> {
    simple_sync(config, files).await
    /*if let Some(cr_number) = cr_number {
        let cr_number = cr_number.parse::<usize>()?;
        cr_sync(config, file, cr_number).await
    } else {
        simple_sync(config, files).await
    }*/
}

/*async fn cr_sync(
    config: &fastn::Config,
    files: Option<Vec<String>>,
    cr_number: usize,
) -> fastn::Result<()> {
    let mut cr_workspace = config.get_cr_workspace(cr_number).await?;
    let changed_files = {
        let mut changed_files = config
            .get_files_status_with_workspace(&mut cr_workspace.workspace)
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
}*/

async fn simple_sync(config: &fastn::Config, files: Option<Vec<String>>) -> fastn::Result<()> {
    use itertools::Itertools;

    let mut workspace = config.get_clone_workspace().await?;
    let changed_files = {
        let mut changed_files = config
            .get_files_status_with_workspace(&mut workspace)
            .await?;
        if let Some(ref files) = files {
            changed_files.retain(|v| files.contains(&v.get_file_path()));
        }
        changed_files
    };
    let changed_files = changed_files
        .into_iter()
        .filter_map(|v| v.sync_request(None))
        .collect_vec();

    sync_(config, changed_files, &mut workspace).await?;
    config
        .update_workspace(workspace.into_values().collect_vec())
        .await
}

pub(crate) async fn sync_(
    config: &fastn::Config,
    request_files: Vec<fastn::apis::sync2::SyncRequestFile>,
    workspace: &mut std::collections::BTreeMap<String, fastn::workspace::WorkspaceEntry>,
) -> fastn::Result<()> {
    let history = tokio::fs::read_to_string(config.history_file()).await?;
    let sync_request = fastn::apis::sync2::SyncRequest {
        package_name: config.package.name.to_string(),
        files: request_files,
        history,
    };
    let response = send_to_fastn_serve(&sync_request).await?;
    update_current_directory(config, &response).await?;
    update_history(config, &response.dot_history, &response.latest_ftd).await?;
    update_workspace(&response, workspace).await?;
    Ok(())
}

async fn update_workspace(
    response: &fastn::apis::sync2::SyncResponse,
    workspace: &mut std::collections::BTreeMap<String, fastn::workspace::WorkspaceEntry>,
) -> fastn::Result<()> {
    use itertools::Itertools;

    let remote_history = fastn::history::FileHistory::from_ftd(response.latest_ftd.as_str())?;
    let remote_manifest =
        fastn::history::FileHistory::get_remote_manifest(remote_history.as_slice(), true)?;
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
    for (file, file_edit) in remote_manifest.into_iter() {
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
    config: &fastn::Config,
    files: &[fastn::apis::sync2::File],
    latest_ftd: &str,
) -> fastn::Result<()> {
    for file in files {
        fastn::utils::update(
            &config.remote_history_dir().join(file.path.as_str()),
            &file.content,
        )
        .await?;
    }
    fastn::utils::update(&config.history_file(), latest_ftd.as_bytes()).await?;
    Ok(())
}

async fn update_current_directory(
    config: &fastn::Config,
    response: &fastn::apis::sync2::SyncResponse,
) -> fastn::Result<()> {
    for file in response.files.iter() {
        match file {
            fastn::apis::sync2::SyncResponseFile::Add {
                path,
                content,
                status,
            } => {
                if status.add_add_conflict() {
                    println!("CloneAddedRemoteAdded: {}", path);
                } else {
                    fastn::utils::update(&config.root.join(path), content).await?;
                }
            }
            fastn::apis::sync2::SyncResponseFile::Update {
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
                    fastn::utils::update(&config.root.join(path), content).await?;
                }
            }
            fastn::apis::sync2::SyncResponseFile::Delete { path, .. } => {
                if config.root.join(path).exists() {
                    tokio::fs::remove_file(config.root.join(path)).await?;
                }
            }
        }
    }
    Ok(())
}

async fn send_to_fastn_serve(
    data: &fastn::apis::sync2::SyncRequest,
) -> fastn::Result<fastn::apis::sync2::SyncResponse> {
    #[derive(serde::Deserialize, std::fmt::Debug)]
    struct ApiResponse {
        message: Option<String>,
        data: Option<fastn::apis::sync2::SyncResponse>,
        success: bool,
    }

    let response: ApiResponse = crate::http::post_json(
        "http://127.0.0.1:8000/-/sync2/",
        serde_json::to_string(&data)?,
    )
    .await?;

    if !response.success {
        return Err(fastn::Error::APIResponseError(
            response
                .message
                .unwrap_or_else(|| "Some Error occurred".to_string()),
        ));
    }

    match response.data {
        Some(data) => Ok(data),
        None => Err(fastn::Error::APIResponseError(
            "Unexpected API behaviour".to_string(),
        )),
    }
}
