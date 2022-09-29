pub async fn revert(config: &fpm::Config, path: &str) -> fpm::Result<()> {
    use itertools::Itertools;

    let mut workspace = config.get_workspace_map().await?;
    let get_files_status = config
        .get_files_status_with_workspace(&mut workspace)
        .await?;
    let file_status =
        if let Some(file_status) = get_files_status.iter().find(|v| v.get_file_path().eq(path)) {
            file_status
        } else {
            config
                .write_workspace(workspace.into_values().collect_vec().as_slice())
                .await?;
            return Err(fpm::Error::UsageError {
                message: format!("{} not found", path),
            });
        };

    if let Some(server_version) = file_status.get_latest_version() {
        let server_path = config.history_path(path, server_version);
        fpm::utils::copy(&server_path, &config.root.join(path)).await?;
        if let Some(workspace_entry) = workspace.get_mut(path) {
            workspace_entry.version = Some(server_version);
            workspace_entry.deleted = None;
        }
    } else {
        // in case of new file added
        tokio::fs::remove_file(path).await?;
        workspace.remove(path);
    }
    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;
    Ok(())
}

/*pub async fn revert_(config: &fpm::Config, path: &str) -> fpm::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fpm::snapshot::get_workspace(config).await?;
    if let Some(workspace) = workspaces.get_mut(path) {
        if workspace
            .workspace
            .eq(&fpm::snapshot::WorkspaceType::CloneEditedRemoteDeleted)
        {
            if config.root.join(path).exists() {
                tokio::fs::remove_file(config.root.join(path)).await?;
            }
        } else {
            let revert_path =
                fpm::utils::history_path(path, config.root.as_str(), &workspace.conflicted);
            tokio::fs::copy(revert_path, config.root.join(path)).await?;
        }
        workspace.set_revert();
    } else {
        let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;
        if let Some(timestamp) = snapshots.get(path) {
            let revert_path = fpm::utils::history_path(path, config.root.as_str(), timestamp);

            fpm::utils::update1(
                &config.root,
                path,
                tokio::fs::read(revert_path).await?.as_slice(),
            )
            .await?;
        }
    }

    if workspaces.is_empty() {
        fpm::snapshot::create_workspace(config, workspaces.into_values().collect_vec().as_slice())
            .await?;
    }

    Ok(())
}*/
