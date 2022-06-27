pub async fn mark_resolve(config: &fpm::Config, path: &str) -> fpm::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fpm::snapshot::get_workspace(config).await?;
    workspaces.remove(path);
    fpm::snapshot::create_workspace(config, workspaces.into_values().collect_vec().as_slice())
        .await?;
    tokio::fs::remove_file(config.conflicted_dir().join(path)).await?;
    Ok(())
}
