pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
    document_id: &str,
    base_url: &str,
) -> ftd::p1::Result<ftd::Value> {
    let versions =
        futures::executor::block_on(config.get_versions(&config.package)).map_err(|e| {
            ftd::p1::Error::ParseError {
                message: format!("Cant find versions: {:?}", e),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            }
        })?;

    let version = if let Some(number) = document_id
        .split_once('/')
        .map(|(v, _)| v.strip_prefix('v'))
        .flatten()
    {
        number
            .parse::<i32>()
            .map_err(|_| ftd::p1::Error::ParseError {
                message: format!("Incorrect version `{:?}` in `{:?}`", number, document_id),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            })?
    } else {
        // 0 is base version
        0
    };

    let doc_id = if let Some(doc) = document_id.split_once('/').map(|(_, v)| v) {
        doc
    } else {
        document_id
    }
    .to_string();

    let base_url = base_url
        .trim_end_matches('/')
        .trim_start_matches('/')
        .to_string();
    let base_url = if !base_url.is_empty() {
        format!("/{base_url}/")
    } else {
        String::from("/")
    };

    let url = match doc_id.as_str().rsplit_once('.') {
        Some(("index", "ftd")) => {
            // Index.ftd found. Return index.html
            format!("{base_url}index.html")
        }
        Some((file_path, "ftd")) | Some((file_path, "md")) => {
            format!("{base_url}{file_path}/index.html")
        }
        Some(_) | None => {
            // Unknown file found, create URL
            format!(
                "{base_url}{file_path}/index.html",
                file_path = doc_id.as_str()
            )
        }
    };

    let mut version_toc = "".to_string();
    for (k, v) in versions {
        if [version, 0].contains(&k) {
            continue;
        }
        if v.iter().map(|v| v.get_id()).any(|x| x == doc_id) {
            version_toc = format!("{}- v{}: v{}{}", version_toc, k, k, url);
        }
    }

    let toc_items = fpm::library::toc::ToC::parse(version_toc.as_str(), doc.name)
        .map_err(|e| ftd::p1::Error::ParseError {
            message: format!("Cannot parse body: {:?}", e),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?
        .items
        .iter()
        .map(|item| item.to_toc_item_compat())
        .collect::<Vec<fpm::library::toc::TocItemCompat>>();

    doc.from_json(&toc_items, section)
}
