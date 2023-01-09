pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    if let Some(ref sitemap) = config.package.sitemap {
        return doc.from_json(
            &fpm::library2022::processor::sitemap::to_sitemap_compat(sitemap),
            section,
        );
    }
    doc.from_json(
        &fpm::library2022::processor::sitemap::SiteMapCompat::default(),
        section,
    )
}
