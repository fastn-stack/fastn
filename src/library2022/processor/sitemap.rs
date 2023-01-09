pub fn process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
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
            return doc.from_json(&sitemap, &kind, value.line_number());
        }
    }
    doc.from_json(
        &fpm::sitemap::SiteMapCompat::default(),
        &kind,
        value.line_number(),
    )
}

pub fn full_sitemap_process<'a>(
    value: ftd::ast::VariableValue,
    kind: ftd::interpreter2::Kind,
    doc: &ftd::interpreter2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::interpreter2::Result<ftd::interpreter2::Value> {
    if let Some(ref sitemap) = config.package.sitemap {
        return doc.from_json(&to_sitemap_compat(sitemap), &kind, value.line_number());
    }
    doc.from_json(
        &fpm::sitemap::SiteMapCompat::default(),
        &kind,
        value.line_number(),
    )
}

#[derive(Default, Debug, serde::Serialize)]
pub struct TocItemCompat {
    pub id: String,
    pub title: Option<String>,
    pub bury: bool,
    #[serde(rename = "extra-data")]
    pub extra_data: Vec<fpm::library2022::KeyValueData>,
    #[serde(rename = "is-active")]
    pub is_active: bool,
    #[serde(rename = "nav-title")]
    pub nav_title: Option<String>,
    pub children: Vec<fpm::sitemap::toc::TocItemCompat>,
    pub skip: bool,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

#[derive(Default, Debug, serde::Serialize)]
pub struct SubSectionCompat {
    pub id: Option<String>,
    pub title: Option<String>,
    pub bury: bool,
    pub visible: bool,
    #[serde(rename = "extra-data")]
    pub extra_data: Vec<fpm::library2022::KeyValueData>,
    #[serde(rename = "is-active")]
    pub is_active: bool,
    #[serde(rename = "nav-title")]
    pub nav_title: Option<String>,
    pub toc: Vec<TocItemCompat>,
    pub skip: bool,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

#[derive(Default, Debug, serde::Serialize)]
pub struct SectionCompat {
    id: String,
    title: Option<String>,
    bury: bool,
    #[serde(rename = "extra-data")]
    extra_data: Vec<fpm::library2022::KeyValueData>,
    #[serde(rename = "is-active")]
    is_active: bool,
    #[serde(rename = "nav-title")]
    nav_title: Option<String>,
    subsections: Vec<SubSectionCompat>,
    readers: Vec<String>,
    writers: Vec<String>,
}

#[derive(Default, Debug, serde::Serialize)]
pub struct SiteMapCompat {
    sections: Vec<SectionCompat>,
    readers: Vec<String>,
    writers: Vec<String>,
}

pub fn to_sitemap_compat(sitemap: &fpm::sitemap::Sitemap) -> fpm::sitemap::SiteMapCompat {
    use itertools::Itertools;
    fn to_toc_compat(toc_item: &fpm::sitemap::toc::TocItem) -> fpm::sitemap::toc::TocItemCompat {
        let toc_compat = fpm::sitemap::toc::TocItemCompat {
            url: Some(toc_item.id.clone()),
            number: None,
            title: toc_item.title.clone(),
            path: None,
            is_heading: false,
            font_icon: None,
            bury: toc_item.bury,
            extra_data: toc_item.extra_data.to_owned(),
            is_active: toc_item.is_active,
            is_open: false,
            nav_title: toc_item.nav_title.clone(),
            children: toc_item.children.iter().map(to_toc_compat).collect_vec(),
            readers: toc_item.readers.clone(),
            writers: toc_item.writers.clone(),
            is_disabled: false,
            image_src: None,
            document: None,
        };
        toc_compat
    }

    fn to_subsection_compat(
        subsection: &fpm::sitemap::section::Subsection,
    ) -> fpm::sitemap::toc::TocItemCompat {
        fpm::sitemap::toc::TocItemCompat {
            url: subsection.id.clone(),
            title: subsection.title.clone(),
            path: None,
            is_heading: false,
            font_icon: None,
            bury: subsection.bury,
            extra_data: subsection.extra_data.to_owned(),
            is_active: subsection.is_active,
            is_open: false,
            image_src: None,
            nav_title: subsection.nav_title.clone(),
            children: subsection.toc.iter().map(to_toc_compat).collect_vec(),
            readers: subsection.readers.clone(),
            writers: subsection.writers.clone(),
            number: None,
            is_disabled: false,
            document: None,
        }
    }

    fn to_section_compat(
        section: &fpm::sitemap::section::Section,
    ) -> fpm::sitemap::toc::TocItemCompat {
        fpm::sitemap::toc::TocItemCompat {
            url: Some(section.id.to_string()),
            number: None,
            title: section.title.clone(),
            path: None,
            is_heading: false,
            font_icon: None,
            bury: section.bury,
            extra_data: section.extra_data.to_owned(),
            is_active: section.is_active,
            is_open: false,
            nav_title: section.nav_title.clone(),
            children: section
                .subsections
                .iter()
                .map(to_subsection_compat)
                .collect_vec(),
            readers: section.readers.clone(),
            writers: section.writers.clone(),
            is_disabled: false,
            image_src: None,
            document: None,
        }
    }

    fpm::sitemap::SiteMapCompat {
        sections: sitemap.sections.iter().map(to_section_compat).collect_vec(),
        subsections: vec![],
        toc: vec![],
        current_section: None,
        current_subsection: None,
        current_page: None,
        readers: sitemap.readers.clone(),
        writers: sitemap.writers.clone(),
    }
}
