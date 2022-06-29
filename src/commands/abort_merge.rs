pub async fn abort_merge(config: &fpm::Config, path: &str) -> fpm::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fpm::snapshot::get_workspace(config).await?;
    if let Some(workspace) = workspaces.get_mut(path) {
        if workspace
            .workspace
            .eq(&fpm::snapshot::WorkspaceType::ClientDeletedServerEdited)
        {
            if config.root.join(path).exists() {
                tokio::fs::remove_file(config.root.join(path)).await?;
            }
        } else {
            tokio::fs::copy(config.conflicted_dir().join(path), config.root.join(path)).await?;
        }
        workspace.set_abort();
    }
    fpm::snapshot::create_workspace(config, workspaces.into_values().collect_vec().as_slice())
        .await?;

    Ok(())
}
