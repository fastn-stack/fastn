pub(crate) async fn construct_tree(
    config: &fastn_core::Config,
    files: &[String],
    snapshots: &std::collections::BTreeMap<String, u128>,
    workspaces: &std::collections::BTreeMap<String, fastn_core::snapshot::Workspace>,
) -> fastn_core::Result<Vec<fastn_core::sitemap::toc::TocItemCompat>> {
    let mut tree = vec![];
    for file in files {
        insert(
            config,
            &mut tree,
            file,
            format!("-/view-src/{}", file.trim_start_matches('/')).as_str(),
            file,
            snapshots,
            workspaces,
        )
        .await?;
    }
    Ok(tree)
}

#[async_recursion::async_recursion(?Send)]
async fn insert(
    config: &fastn_core::Config,
    tree: &mut Vec<fastn_core::sitemap::toc::TocItemCompat>,
    path: &str,
    url: &str,
    full_path: &str,
    snapshots: &std::collections::BTreeMap<String, u128>,
    workspaces: &std::collections::BTreeMap<String, fastn_core::snapshot::Workspace>,
) -> fastn_core::Result<()> {
    let (path, rest) = if let Some((path, rest)) = path.split_once('/') {
        (path, Some(rest))
    } else {
        (path, None)
    };

    let node = if let Some(node) = tree
        .iter_mut()
        .find(|node| node.title.as_ref().map(|v| v.eq(path)).unwrap_or(false))
    {
        node
    } else {
        let full_path = rest
            .map(|v| full_path.trim_end_matches(v))
            .unwrap_or(full_path);
        tree.push(
            fastn_core::sitemap::toc::TocItemCompat::new(
                None,
                Some(path.to_string()),
                false,
                false,
                vec![],
                vec![],
                None,
                false,
            )
            .add_path(full_path),
        );
        tree.last_mut().unwrap()
    };

    if let Some(rest) = rest {
        insert(
            config,
            &mut node.children,
            rest,
            url,
            full_path,
            snapshots,
            workspaces,
        )
        .await?;
    } else if let Ok(file) = fastn_core::get_file(
        &config.ds,
        config.package.name.to_string(),
        &config.ds.root().join(full_path),
        config.ds.root(),
    )
    .await
    {
        let status =
            fastn_core::commands::status::get_file_status(config, &file, snapshots, workspaces)
                .await?;
        node.url = Some(url.to_string());
        node.number = Some(format!("{:?}", status))
    } else {
        node.number = Some(format!(
            "{:?}",
            fastn_core::commands::status::FileStatus::Deleted
        ))
    }

    Ok(())
}
