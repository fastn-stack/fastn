pub async fn edit(config: &fastn_core::Config, file: &str, cr: &str) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let cr = cr.parse::<usize>()?;

    if !fastn_core::cr::is_open_cr_exists(config, cr).await? {
        return fastn_core::usage_error(format!("CR#{} is closed", cr));
    };

    let cr_track_path = config.cr_track_path(&config.root.join(file), cr);
    let cr_file_path = config.cr_path(cr).join(file);
    if cr_track_path.exists() && cr_file_path.exists() {
        return fastn_core::usage_error(format!("{} is already tracked in cr {}", file, cr));
    }

    let remote_manifest = config.get_remote_manifest(false).await?;
    let file_edit = if let Some(file_edit) = remote_manifest.get(file) {
        file_edit
    } else {
        return Err(fastn_core::Error::UsageError {
            message: format!(
                "{} is not present in remote manifest. Help: Use `fastn add {} --cr {}",
                file, file, cr
            ),
        });
    };

    // copy file to cr directory
    let file_path = config.history_path(file, file_edit.version);

    if cr_file_path.exists() {
        return Err(fastn_core::Error::UsageError {
            message: format!("{} is already exists", cr_file_path),
        });
    }

    if file_path.exists() {
        let content = tokio::fs::read(&file_path).await?;
        fastn_core::utils::update(&cr_file_path, content.as_slice()).await?;
    } else {
        fastn_core::utils::update(&cr_file_path, vec![].as_slice()).await?;
    }

    // tracks the file
    let tracking_info = fastn_core::track::TrackingInfo::new(file, file_edit.version, None);
    fastn_core::track::create_tracking_info(config, &[tracking_info], &cr_file_path).await?;

    // create workspace entry for file and for track
    let mut workspace = config.get_workspace_map().await?;

    workspace.insert(
        config.path_without_root(&cr_file_path)?,
        fastn_core::workspace::WorkspaceEntry {
            filename: config.path_without_root(&cr_file_path)?,
            deleted: None,
            version: None,
            cr: Some(cr),
        },
    );

    workspace.insert(
        config.path_without_root(&config.track_path(&cr_file_path))?,
        fastn_core::workspace::WorkspaceEntry {
            filename: config.path_without_root(&config.track_path(&cr_file_path))?,
            deleted: None,
            version: None,
            cr: None,
        },
    );

    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;

    Ok(())
}
