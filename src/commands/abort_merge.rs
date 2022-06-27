pub async fn abort_merge(config: &fpm::Config, path: &str) -> fpm::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fpm::snapshot::get_workspace(config).await?;
    if let Some(workspace) = workspaces.get_mut(path) {
        workspace.set_abort();
    }
    fpm::snapshot::create_workspace(config, workspaces.into_values().collect_vec().as_slice())
        .await?;
    tokio::fs::copy(config.conflicted_dir().join(path), config.root.join(path)).await?;
    Ok(())
}
