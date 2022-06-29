pub async fn revert(config: &fpm::Config, path: &str) -> fpm::Result<()> {
    use itertools::Itertools;

    let mut workspaces = fpm::snapshot::get_workspace(config).await?;
    if let Some(workspace) = workspaces.get_mut(path) {
        if workspace
            .workspace
            .eq(&fpm::snapshot::WorkspaceType::ClientEditedServerDeleted)
        {
            if config.root.join(path).exists() {
                tokio::fs::remove_file(config.root.join(path)).await?;
            }
        } else {
            let revert_path =
                fpm::utils::history_path(path, config.root.as_str(), &workspace.conflicted);
            tokio::fs::copy(revert_path, config.root.join(path)).await?;
        }
        workspace.set_revert();
    }
    fpm::snapshot::create_workspace(config, workspaces.into_values().collect_vec().as_slice())
        .await?;

    Ok(())
}
