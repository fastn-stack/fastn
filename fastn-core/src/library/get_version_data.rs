use itertools::Itertools;

pub async fn processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fastn_core::Config,
    document_id: &str,
    base_url: &str,
) -> ftd::p1::Result<ftd::Value> {
    let versions =
        config
            .get_versions(&config.package)
            .await
            .map_err(|e| ftd::p1::Error::ParseError {
                message: format!("Cant find versions: {:?}", e),
                doc_id: doc.name.to_string(),
                line_number: section.line_number,
            })?;

    let version = if let Some((v, _)) = document_id.split_once('/') {
        fastn_core::Version::parse(v).map_err(|e| ftd::p1::Error::ParseError {
            message: format!("{:?}", e),
            doc_id: doc.name.to_string(),
            line_number: section.line_number,
        })?
    } else {
        fastn_core::Version::base()
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
        Some(("index", "ftd")) => base_url,
        Some((file_path, "ftd")) | Some((file_path, "md")) => {
            format!("{base_url}{file_path}/")
        }
        Some(_) | None => {
            // Unknown file found, create URL
            format!("{base_url}{file_path}/", file_path = doc_id.as_str())
        }
    };
    let mut found = false;
    if let Some(doc) = versions.get(&fastn_core::Version::base()) {
        if doc.iter().map(|v| v.get_id()).any(|x| x == doc_id) {
            found = true;
        }
    }

    let mut version_toc = vec![];
    for key in versions.keys().sorted() {
        if key.eq(&fastn_core::Version::base()) {
            continue;
        }
        let doc = versions[key].to_owned();
        if !found {
            if !doc.iter().map(|v| v.get_id()).any(|x| x == doc_id) {
                continue;
            }
            found = true;
        }
        version_toc.push(fastn_core::library::toc::TocItem {
            id: None,
            description: None,
            title: Some(key.original.to_string()),
            url: Some(format!("{}{}", key.original, url)),
            path: None,
            bury: false,
            number: vec![],
            is_heading: version.eq(key),
            is_disabled: false,
            img_src: None,
            font_icon: None,
            children: vec![],
            document: None,
        });
    }

    let toc_items = version_toc
        .iter()
        .map(|item| item.to_toc_item_compat())
        .collect::<Vec<fastn_core::library::toc::TocItemCompat>>();

    doc.from_json(&toc_items, section)
}
