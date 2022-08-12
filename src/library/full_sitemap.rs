use itertools::Itertools;

#[derive(Default, Debug, serde::Serialize)]
pub struct TocItemCompat {
    pub id: String,
    pub title: Option<String>,
    #[serde(rename = "extra-data")]
    pub extra_data: Vec<KeyValueData>,
    #[serde(rename = "is-active")]
    pub is_active: bool,
    #[serde(rename = "nav-title")]
    pub nav_title: Option<String>,
    pub children: Vec<TocItemCompat>,
    pub skip: bool,
    pub readers: Vec<String>,
    pub writers: Vec<String>,
}

#[derive(Default, Debug, serde::Serialize)]
struct SubSectionCompat {
    pub id: Option<String>,
    pub title: Option<String>,
    pub visible: bool,
    #[serde(rename = "extra-data")]
    pub extra_data: Vec<KeyValueData>,
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
struct SectionCompat {
    id: String,
    title: Option<String>,
    #[serde(rename = "extra-data")]
    extra_data: Vec<KeyValueData>,
    #[serde(rename = "is-active")]
    is_active: bool,
    #[serde(rename = "nav-title")]
    nav_title: Option<String>,
    subsections: Vec<SubSectionCompat>,
    readers: Vec<String>,
    writers: Vec<String>,
}

#[derive(Default, Debug, serde::Serialize)]
pub struct KeyValueData {
    pub key: String,
    pub value: String,
}

impl KeyValueData {
    pub fn from(key: String, value: String) -> Self {
        Self { key, value }
    }
}

#[derive(Default, Debug, serde::Serialize)]
struct SiteMapCompat {
    sections: Vec<SectionCompat>,
    readers: Vec<String>,
    writers: Vec<String>,
}

pub fn processor(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    if let Some(ref sitemap) = config.package.sitemap {
        return doc.from_json(&to_sitemap_compat(sitemap), section);
    }
    doc.from_json(&SiteMapCompat::default(), section)
}

fn to_sitemap_compat(sitemap: &fpm::sitemap::Sitemap) -> SiteMapCompat {
    fn to_key_value_data(key: &str, value: &str) -> KeyValueData {
        KeyValueData {
            key: key.to_string(),
            value: value.to_string(),
        }
    }

    fn to_toc_compat(toc_item: &fpm::sitemap::TocItem) -> TocItemCompat {
        let toc_compat = TocItemCompat {
            id: toc_item.id.clone(),
            title: toc_item.title.clone(),
            extra_data: toc_item
                .extra_data
                .iter()
                .map(|(k, v)| to_key_value_data(k, v))
                .collect(),
            is_active: toc_item.is_active,
            nav_title: toc_item.nav_title.clone(),
            children: toc_item.children.iter().map(to_toc_compat).collect_vec(),
            skip: toc_item.skip,
            readers: toc_item.readers.clone(),
            writers: toc_item.writers.clone(),
        };
        toc_compat
    }

    fn to_subsection_compat(subsection: &fpm::sitemap::Subsection) -> SubSectionCompat {
        SubSectionCompat {
            id: subsection.id.clone(),
            title: subsection.title.clone(),
            visible: subsection.visible,
            extra_data: subsection
                .extra_data
                .iter()
                .map(|(k, v)| to_key_value_data(k, v))
                .collect(),
            is_active: subsection.is_active,
            nav_title: subsection.nav_title.clone(),
            toc: subsection.toc.iter().map(to_toc_compat).collect_vec(),
            skip: subsection.skip,
            readers: subsection.readers.clone(),
            writers: subsection.writers.clone(),
        }
    }

    fn to_section_compat(section: &fpm::sitemap::Section) -> SectionCompat {
        SectionCompat {
            id: section.id.to_string(),
            title: section.title.clone(),
            extra_data: section
                .extra_data
                .iter()
                .map(|(k, v)| to_key_value_data(k, v))
                .collect(),
            is_active: section.is_active,
            nav_title: section.nav_title.clone(),
            subsections: section
                .subsections
                .iter()
                .map(to_subsection_compat)
                .collect_vec(),
            readers: section.readers.clone(),
            writers: section.writers.clone(),
        }
    }

    SiteMapCompat {
        sections: sitemap.sections.iter().map(to_section_compat).collect_vec(),
        readers: sitemap.readers.clone(),
        writers: sitemap.writers.clone(),
    }
}
