pub async fn abort_merge(config: &fastn::Config, path: &str) -> fastn::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fastn::snapshot::get_workspace(config).await?;
    if let Some(workspace) = workspaces.get_mut(path) {
        if workspace
            .workspace
            .eq(&fastn::snapshot::WorkspaceType::CloneDeletedRemoteEdited)
        {
            if config.root.join(path).exists() {
                tokio::fs::remove_file(config.root.join(path)).await?;
            }
        } else {
            tokio::fs::copy(config.conflicted_dir().join(path), config.root.join(path)).await?;
        }
        workspace.set_abort();
    }
    fastn::snapshot::create_workspace(config, workspaces.into_values().collect_vec().as_slice())
        .await?;

    Ok(())
}
