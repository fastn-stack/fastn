use itertools::Itertools;

pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    processor_(section, doc, config).map_err(|e| ftd::p1::Error::ParseError {
        message: e.to_string(),
        doc_id: doc.name.to_string(),
        line_number: section.line_number,
    })
}

pub fn processor_(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> fpm::Result<ftd::Value> {
    let root = config.get_root_for_package(&config.package);
    let files = config
        .get_all_file_paths(&config.package, true)?
        .into_iter()
        .filter(|v| v.is_file())
        .map(|v| {
            v.strip_prefix(&root)
                .unwrap_or_else(|_| v.as_path())
                .to_string()
                .replace(std::path::MAIN_SEPARATOR.to_string().as_str(), "/")
        })
        .collect_vec();
    let tree = construct_tree(files.as_slice())?;
    Ok(doc.from_json(&tree, section)?)
}

fn construct_tree(files: &[String]) -> fpm::Result<Vec<fpm::sitemap::TocItemCompat>> {
    let mut tree = vec![];
    for file in files {
        insert(
            &mut tree,
            file,
            format!("-/view-src/{}", file.trim_start_matches('/')).as_str(),
        );
    }
    Ok(tree)
}

fn insert(tree: &mut Vec<fpm::sitemap::TocItemCompat>, path: &str, url: &str) {
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
        tree.push(fpm::sitemap::TocItemCompat::new(
            None,
            Some(path.to_string()),
            false,
            false,
        ));
        tree.last_mut().unwrap()
    };

    if let Some(rest) = rest {
        insert(&mut node.children, rest, url);
    } else {
        node.url = Some(url.to_string());
    }
}
