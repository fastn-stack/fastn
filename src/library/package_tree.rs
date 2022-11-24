use itertools::Itertools;

pub async fn processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    processor_(section, doc, config)
        .await
        .map_err(|e| ftd::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })
}

pub fn processor_sync<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    futures::executor::block_on(processor_(section, doc, config)).map_err(|e| {
        ftd::p1::Error::ParseError {
            message: e.to_string(),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        }
    })
}

pub async fn processor_<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> fpm::Result<ftd::Value> {
    let root = config.get_root_for_package(&config.package);
    let snapshots = fpm::snapshot::get_latest_snapshots(&config.root).await?;
    let workspaces = fpm::snapshot::get_workspace(config).await?;
    let all_files = config
        .get_files(&config.package)
        .await?
        .into_iter()
        .map(|v| v.get_id())
        .collect_vec();
    let deleted_files = snapshots
        .keys()
        .filter(|v| !all_files.contains(v))
        .map(|v| v.to_string());

    let mut files = config
        .get_all_file_paths1(&config.package, true)?
        .into_iter()
        .filter(|v| v.is_file())
        .map(|v| {
            v.strip_prefix(&root)
                .unwrap_or_else(|_| v.as_path())
                .to_string()
                .replace(std::path::MAIN_SEPARATOR.to_string().as_str(), "/")
        })
        .collect_vec();
    files.extend(deleted_files);

    let tree = construct_tree(config, files.as_slice(), &snapshots, &workspaces).await?;
    Ok(doc.from_json(&tree, section)?)
}

async fn construct_tree(
    config: &fpm::Config,
    files: &[String],
    snapshots: &std::collections::BTreeMap<String, u128>,
    workspaces: &std::collections::BTreeMap<String, fpm::snapshot::Workspace>,
) -> fpm::Result<Vec<fpm::sitemap::toc::TocItemCompat>> {
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
    config: &fpm::Config,
    tree: &mut Vec<fpm::sitemap::toc::TocItemCompat>,
    path: &str,
    url: &str,
    full_path: &str,
    snapshots: &std::collections::BTreeMap<String, u128>,
    workspaces: &std::collections::BTreeMap<String, fpm::snapshot::Workspace>,
) -> fpm::Result<()> {
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
            fpm::sitemap::toc::TocItemCompat::new(
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
    } else if let Ok(file) = fpm::get_file(
        config.package.name.to_string(),
        &config.root.join(full_path),
        &config.root,
    )
    .await
    {
        let status = fpm::commands::status::get_file_status(&file, snapshots, workspaces).await?;
        node.url = Some(url.to_string());
        node.number = Some(format!("{:?}", status))
    } else {
        node.number = Some(format!("{:?}", fpm::commands::status::FileStatus::Deleted))
    }

    Ok(())
}
