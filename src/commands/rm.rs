use itertools::Itertools;

pub async fn rm(config: &fpm::Config, file: &str) -> fpm::Result<()> {
    let mut workspace: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> = config
        .read_workspace()
        .await?
        .into_iter()
        .map(|v| (v.filename.to_string(), v))
        .collect();

    if !config
        .get_latest_file_paths()
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
