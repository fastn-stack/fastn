use itertools::Itertools;

pub async fn add(config: &fpm::Config, file: &str, cr: Option<&str>) -> fpm::Result<()> {
    if let Some(cr) = cr {
        let cr = cr.parse::<usize>()?;
        cr_add(config, file, cr).await
    } else {
        simple_add(config, file).await
    }
}

async fn simple_add(config: &fpm::Config, file: &str) -> fpm::Result<()> {
    let mut workspace = config.get_clone_workspace().await?;

    if workspace.contains_key(file) {
        return Err(fpm::Error::UsageError {
            message: format!("{} is already in workspace", file),
        });
    }

    if !config.root.join(file).exists() {
        return Err(fpm::Error::UsageError {
            message: format!("{} doesn't exists", file),
        });
    }

    workspace.insert(
        file.to_string(),
        fpm::workspace::WorkspaceEntry {
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

async fn cr_add(config: &fpm::Config, file: &str, cr: usize) -> fpm::Result<()> {
    if !fpm::cr::is_open_cr_exists(config, cr).await? {
        return fpm::usage_error(format!("CR#{} is closed", cr));
    };
    let remote_manifest = config.get_remote_manifest(false).await?;
    if remote_manifest.contains_key(file) {
        return Err(fpm::Error::UsageError {
            message: format!(
                "{} is present in remote manifest. Help: Use `fpm edit {} --cr {}",
                file, file, cr
            ),
        });
    }

    let mut workspace = config.get_workspace_map().await?;

    let cr_file_path = config.cr_path(cr).join(file);
    workspace.insert(
        config.path_without_root(&cr_file_path)?,
        fpm::workspace::WorkspaceEntry {
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
        fpm::utils::copy(&file_path, &cr_file_path).await?;
    } else {
        fpm::utils::update(&cr_file_path, vec![].as_slice()).await?;
    }

    Ok(())
}
