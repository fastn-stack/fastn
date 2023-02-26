pub async fn mark_resolved(config: &fastn_core::Config, path: &str) -> fastn_core::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fastn_core::snapshot::get_workspace(config).await?;
    workspaces.remove(path);

    fastn_core::snapshot::create_workspace(
        config,
        workspaces.into_values().collect_vec().as_slice(),
    )
    .await?;
    // TODO: Check workspace value and then delete it
    // This is certainly bad idea
    if config.conflicted_dir().join(path).exists() {
        tokio::fs::remove_file(config.conflicted_dir().join(path)).await?;
    }
    Ok(())
}
