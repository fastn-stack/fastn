pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    if let Some(ref sitemap) = config.sitemap {
        let doc_id = doc
            .name
            .to_string()
            .replace(config.package.name.as_str(), "");
        if let Some(sitemap) = sitemap.get_sitemap_by_id(doc_id.trim_start_matches('/')) {
            return doc.from_json(&sitemap, section);
        }
    }
    doc.from_json(&fpm::sitemap::SiteMapCompat::default(), section)
}
