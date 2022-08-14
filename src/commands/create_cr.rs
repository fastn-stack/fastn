use itertools::Itertools;

pub async fn create_cr(config: &fpm::Config) -> fpm::Result<()> {
    let cr_number = config.extract_cr_number().await?;
    let cr_about_content = fpm::cr::generate_cr_about_content(&fpm::cr::CRAbout {
        title: format!("CR#{cr_number}"),
        description: Some(format!("Change Request {cr_number}")),
        cr_number: cr_number as usize,
        open: true,
    });
    let cr_about_content = fpm::cr::resolve_cr_about(
        edit::edit(cr_about_content)
            .map_err(|e| fpm::Error::UsageError {
                message: e.to_string(),
            })?
            .as_str(),
        cr_number as usize,
    )
    .await?;
    fpm::cr::create_cr_about(config, &cr_about_content).await?;

    let mut workspace = config.get_workspace_map().await?;
    let filename = config.path_without_root(&config.cr_about_path(cr_about_content.cr_number))?;
    workspace.insert(
        filename.to_string(),
        fpm::workspace::WorkspaceEntry {
            filename,
            deleted: None,
            version: None,
            cr: Some(cr_number as usize),
        },
    );
    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;
    Ok(())
}
