use itertools::Itertools;

pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    if let Some(ref sitemap) = config.package.sitemap {
        let doc_id = config
            .current_document
            .clone()
            .map(|v| fpm::utils::id_to_path(v.as_str()))
            .unwrap_or_else(|| {
                doc.name
                    .to_string()
                    .replace(config.package.name.as_str(), "")
            })
            .trim()
            .replace(std::path::MAIN_SEPARATOR, "/");

        if let Some(sitemap) = sitemap.get_sitemap_by_id(doc_id.as_str()) {
            return doc.from_json(&sitemap, section);
        }
    }
    doc.from_json(&fpm::sitemap::SiteMapCompat::default(), section)
}

pub fn document_readers(
    section: &ftd::p1::Section,
    document_id: &str,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    // TODO: document key should be optional
    let document =
        section
            .header
            .string_with_default(document_id, section.line_number, "document", "/")?;

    let readers = match config.package.sitemap.as_ref() {
        Some(s) => s
            .readers(document.as_str(), &config.package.groups)
            .0
            .into_iter()
            .map(|g| g.to_group_compat())
            .collect_vec(),
        None => vec![],
    };

    doc.from_json(&readers, section)
}

pub fn document_writers(
    section: &ftd::p1::Section,
    document_id: &str,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    // TODO: document key should be optional
    let document =
        section
            .header
            .string_with_default(document_id, section.line_number, "document", "/")?;

    let writers = match config.package.sitemap.as_ref() {
        Some(s) => s
            .writers(document.as_str(), &config.package.groups)
            .into_iter()
            .map(|g| g.to_group_compat())
            .collect_vec(),
        None => vec![],
    };

    doc.from_json(&writers, section)
}
