pub async fn create_cr(config: &fastn_core::Config, title: Option<&str>) -> fastn_core::Result<()> {
    let cr_number = config.extract_cr_number().await?;
    let cr_meta_content = fastn_core::cr::generate_cr_meta_content(&fastn_core::cr::CRMeta {
        title: title
            .map(ToString::to_string)
            .unwrap_or(format!("CR#{cr_number}")),
        cr_number: cr_number as usize,
        open: true,
    });
    /*let cr_meta_content = fastn_core::cr::resolve_cr_meta(
        edit::edit(cr_meta_content)
            .map_err(|e| fastn_core::Error::UsageError {
                message: e.to_string(),
            })?
            .as_str(),
        cr_number as usize,
    )
    .await?;*/
    let cr_meta_content =
        fastn_core::cr::resolve_cr_meta(cr_meta_content.as_str(), cr_number as usize).await?;
    add_cr_to_workspace(config, &cr_meta_content).await
}

pub(crate) async fn add_cr_to_workspace(
    config: &fastn_core::Config,
    cr_meta: &fastn_core::cr::CRMeta,
) -> fastn_core::Result<()> {
    use itertools::Itertools;

    fastn_core::cr::create_cr_meta(config, cr_meta).await?;
    fastn_core::cr::create_cr_about(config, cr_meta).await?;

    let mut workspace = config.get_workspace_map().await?;
    let cr_meta_filename = config.path_without_root(&config.cr_meta_path(cr_meta.cr_number))?;
    workspace.insert(
        cr_meta_filename.to_string(),
        fastn_core::workspace::WorkspaceEntry {
            filename: cr_meta_filename,
            deleted: None,
            version: None,
            cr: Some(cr_meta.cr_number),
        },
    );

    let cr_about_filename = config.path_without_root(&config.cr_about_path(cr_meta.cr_number))?;
    workspace.insert(
        cr_about_filename.to_string(),
        fastn_core::workspace::WorkspaceEntry {
            filename: cr_about_filename,
            deleted: None,
            version: None,
            cr: Some(cr_meta.cr_number),
        },
    );
    config
        .write_workspace(workspace.into_values().collect_vec().as_slice())
        .await?;
    Ok(())
}
