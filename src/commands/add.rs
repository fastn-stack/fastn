use itertools::Itertools;

pub async fn add(config: &fpm::Config, file: &str) -> fpm::Result<()> {
    let mut workspace: std::collections::BTreeMap<String, fpm::workspace::WorkspaceEntry> = config
        .read_workspace()
        .await?
        .into_iter()
        .map(|v| (v.filename.to_string(), v))
        .collect();
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
        },
    );

    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;

    Ok(())
}
