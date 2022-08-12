use itertools::Itertools;

pub async fn rm(config: &fpm::Config, file: &str, cr: Option<&str>) -> fpm::Result<()> {
    if let Some(cr) = cr {
        let cr = cr.parse::<usize>()?;
        cr_rm(config, file, cr).await
    } else {
        simple_rm(config, file).await
    }
}

async fn simple_rm(config: &fpm::Config, file: &str) -> fpm::Result<()> {
    let mut workspace: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> = config
        .read_workspace()
        .await?
        .into_iter()
        .map(|v| (v.filename.to_string(), v))
        .collect();

    if !config
        .get_non_deleted_latest_file_paths()
        .await?
        .iter()
        .any(|(v, _)| v.eq(file))
    {
        return Err(fpm::Error::UsageError {
            message: format!("{} doesn't exists in latest. If added in workspace use `fpm revert {}` command instead", file, file),
        });
    }

    if let Some(workspace_entry) = workspace.get_mut(file) {
        workspace_entry.set_deleted();
        let path = config.root.join(&workspace_entry.filename);
        if path.exists() {
            tokio::fs::remove_file(path).await?;
        }
    } else {
        return Err(fpm::Error::UsageError {
            message: format!("{} is not present in workspace", file),
        });
    }

    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;

    Ok(())
}

async fn cr_rm(config: &fpm::Config, file: &str, cr: usize) -> fpm::Result<()> {
    // create workspace entry
    let mut workspace: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> = config
        .read_workspace()
        .await?
        .into_iter()
        .map(|v| (v.filename.to_string(), v))
        .collect();

    let remote_manifest = config.get_remote_manifest().await?;
    let file_edit = if let Some(file_edit) = remote_manifest.get(file) {
        file_edit
    } else {
        return Err(fpm::Error::UsageError {
            message: format!("{} is not present in remote manifest.", file,),
        });
    };

    let cr_file_path = config.cr_path(cr).join(file);
    workspace.insert(
        file.to_string(),
        fpm::workspace::WorkspaceEntry {
            filename: cr_file_path.to_string(),
            deleted: Some(true),
            version: Some(file_edit.version),
            cr: Some(cr),
        },
    );

    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;

    // create delete entry
    let mut deleted_files = fpm::cr::get_deleted_files(config, cr).await?;
    if deleted_files.contains(&file.to_string()) {
        return fpm::usage_error(format!("{} is already deleted in CR#{}", file, cr));
    }
    deleted_files.push(file.to_string());

    fpm::cr::create_deleted_files(config, cr, deleted_files.as_slice()).await?;

    Ok(())
}
