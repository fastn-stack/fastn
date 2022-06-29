pub async fn mark_resolve(config: &fpm::Config, path: &str) -> fpm::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fpm::snapshot::get_workspace(config).await?;
    workspaces.remove(path);

    fpm::snapshot::create_workspace(config, workspaces.into_values().collect_vec().as_slice())
        .await?;
    // TODO: Check workspace value and then delete it
    // This is certainly bad idea
    if config.conflicted_dir().join(path).exists() {
        tokio::fs::remove_file(config.conflicted_dir().join(path)).await?;
    }
    Ok(())
}
