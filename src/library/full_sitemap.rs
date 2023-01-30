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

        return doc.from_json(
            &fpm::library2022::processor::sitemap::to_sitemap_compat(sitemap, doc_id.as_str()),
            section,
        );
    }
    doc.from_json(
        &fpm::library2022::processor::sitemap::SiteMapCompat::default(),
        section,
    )
}
