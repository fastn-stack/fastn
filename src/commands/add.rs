pub async fn add(config: &fastn::Config, file: &str, cr: Option<&str>) -> fastn::Result<()> {
    if let Some(cr) = cr {
        let cr = cr.parse::<usize>()?;
        cr_add(config, file, cr).await
    } else {
        simple_add(config, file).await
    }
}

async fn simple_add(config: &fastn::Config, file: &str) -> fastn::Result<()> {
    use itertools::Itertools;

    let mut workspace = config.get_clone_workspace().await?;

    if workspace.contains_key(file) {
        return Err(fastn::Error::UsageError {
            message: format!("{} is already in workspace", file),
        });
    }

    if !config.root.join(file).exists() {
        return Err(fastn::Error::UsageError {
            message: format!("{} doesn't exists", file),
        });
    }

    workspace.insert(
        file.to_string(),
        fastn::workspace::WorkspaceEntry {
            filename: file.to_string(),
            deleted: None,
            version: None,
            cr: None,
        },
    );

    config
        .update_workspace(workspace.into_values().collect_vec())
        .await?;

    Ok(())
}

async fn cr_add(config: &fastn::Config, file: &str, cr: usize) -> fastn::Result<()> {
    use itertools::Itertools;

    if !fastn::cr::is_open_cr_exists(config, cr).await? {
        return fastn::usage_error(format!("CR#{} is closed", cr));
    };
    let remote_manifest = config.get_remote_manifest(false).await?;
    if remote_manifest.contains_key(file) {
        return Err(fastn::Error::UsageError {
            message: format!(
                "{} is present in remote manifest. Help: Use `fastn edit {} --cr {}",
                file, file, cr
            ),
        });
    }

    let mut workspace = config.get_workspace_map().await?;

    let cr_file_path = config.cr_path(cr).join(file);
    workspace.insert(
        config.path_without_root(&cr_file_path)?,
        fastn::workspace::WorkspaceEntry {
            filename: config.path_without_root(&cr_file_path)?,
            deleted: None,
            version: None,
            cr: Some(cr),
        },
    );

    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;

    let file_path = config.root.join(file);

    if file_path.exists() {
        fastn::utils::copy(&file_path, &cr_file_path).await?;
    } else {
        fastn::utils::update(&cr_file_path, vec![].as_slice()).await?;
    }

    Ok(())
}
