use itertools::Itertools;

pub async fn edit(config: &fpm::Config, file: &str, cr: &str) -> fpm::Result<()> {
    let cr = cr.parse::<usize>()?;

    if !fpm::cr::is_open_cr_exists(config, cr).await? {
        return fpm::usage_error(format!("CR#{} is closed", cr));
    };

    let cr_track_path = config.cr_track_path(&config.root.join(file), cr);
    let cr_file_path = config.cr_path(cr).join(file);
    if cr_track_path.exists() && cr_file_path.exists() {
        return fpm::usage_error(format!("{} is already tracked in cr {}", file, cr));
    }

    let remote_manifest = config.get_remote_manifest(false).await?;
    let file_edit = if let Some(file_edit) = remote_manifest.get(file) {
        file_edit
    } else {
        return Err(fpm::Error::UsageError {
            message: format!(
                "{} is not present in remote manifest. Help: Use `fpm add {} --cr {}",
                file, file, cr
            ),
        });
    };

    // copy file to cr directory
    let file_path = config.history_path(file, file_edit.version);

    if cr_file_path.exists() {
        return Err(fpm::Error::UsageError {
            message: format!("{} is already exists", cr_file_path),
        });
    }

    if file_path.exists() {
        let content = tokio::fs::read(&file_path).await?;
        fpm::utils::update(&cr_file_path, content.as_slice()).await?;
    } else {
        fpm::utils::update(&cr_file_path, vec![].as_slice()).await?;
    }

    // tracks the file
    let tracking_info = fpm::track::TrackingInfo::new(file, file_edit.version, None);
    fpm::track::create_tracking_info(config, &[tracking_info], &cr_file_path).await?;

    // create workspace entry for file and for track
    let mut workspace = config.get_workspace_map().await?;

    workspace.insert(
        config.path_without_root(&cr_file_path)?,
        fpm::workspace::WorkspaceEntry {
            filename: config.path_without_root(&cr_file_path)?,
            deleted: None,
            version: None,
            cr: Some(cr),
        },
    );

    workspace.insert(
        config.path_without_root(&config.track_path(&cr_file_path))?,
        fpm::workspace::WorkspaceEntry {
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
