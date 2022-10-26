#[derive(Debug, serde::Deserialize, Clone)]
pub struct DynamicUrlsTemp {
    #[serde(rename = "dynamic-urls-body")]
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicUrls {
    pub sections: Vec<fpm::sitemap::section::Section>,
}

impl DynamicUrls {
    pub fn parse(
        global_ids: &std::collections::HashMap<String, String>,
        package_name: &str,
        body: &str,
    ) -> Result<Self, fpm::sitemap::ParseError> {
        // Note: Using Sitemap Parser, because format of dynamic-urls is same as sitemap
        let mut parser = fpm::sitemap::SitemapParser {
            state: fpm::sitemap::ParsingState::WaitingForSection,
            sections: vec![],
            temp_item: None,
            doc_name: package_name.to_string(),
        };

        for line in body.split('\n') {
            parser.read_line(line, global_ids)?;
        }

        if parser.temp_item.is_some() {
            parser.eval_temp_item(global_ids)?;
        }

        Ok(DynamicUrls {
            sections: fpm::sitemap::construct_tree_util(parser.finalize()?),
        })
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_dynamic_urls() {
        let left = fpm::sitemap::DynamicUrls::parse(
            &std::collections::HashMap::new(),
            "abrark.com",
            r#"
# Dynamic Urls Section
- Url 1
  url: /person/<string:name>/
  document: person.ftd
  readers: readers/person
  writers: writers/person
- Url 2
  url: /person/<string:name>/
  document: person.ftd
  readers: readers/person
  writers: writers/person
"#,
        );

        let right = Ok(fpm::sitemap::DynamicUrls {
            sections: vec![fpm::sitemap::section::Section {
                id: "Dynamic Urls Section".to_string(),
                title: Some("Dynamic Urls Section".to_string()),
                file_location: None,
                translation_file_location: None,
                extra_data: Default::default(),
                is_active: false,
                nav_title: None,
                subsections: vec![fpm::sitemap::section::Subsection {
                    id: None,
                    title: None,
                    file_location: None,
                    translation_file_location: None,
                    visible: false,
                    extra_data: Default::default(),
                    is_active: false,
                    nav_title: None,
                    toc: vec![
                        fpm::sitemap::toc::TocItem {
                            id: "/person/<string:name>/".to_string(),
                            title: Some("Url 1".to_string()),
                            file_location: None,
                            translation_file_location: None,
                            extra_data: vec![
                                ("document", "person.ftd"),
                                ("readers", "readers/person"),
                                ("url", "/person/<string:name>/"),
                                ("writers", "writers/person"),
                            ]
                            .into_iter()
                            .map(|(a, b)| (a.to_string(), b.to_string()))
                            .collect(),
                            is_active: false,
                            nav_title: None,
                            children: vec![],
                            skip: true,
                            readers: vec!["readers/person".to_string()],
                            writers: vec!["writers/person".to_string()],
                            document: Some("person.ftd".to_string()),
                            path_parameters: vec![("string".to_string(), "name".to_string())],
                        },
                        fpm::sitemap::toc::TocItem {
                            id: "/person/<string:name>/".to_string(),
                            title: Some("Url 2".to_string()),
                            file_location: None,
                            translation_file_location: None,
                            extra_data: vec![
                                ("document", "person.ftd"),
                                ("readers", "readers/person"),
                                ("url", "/person/<string:name>/"),
                                ("writers", "writers/person"),
                            ]
                            .into_iter()
                            .map(|(a, b)| (a.to_string(), b.to_string()))
                            .collect(),
                            is_active: false,
                            nav_title: None,
                            children: vec![],
                            skip: true,
                            readers: vec!["readers/person".to_string()],
                            writers: vec!["writers/person".to_string()],
                            document: Some("person.ftd".to_string()),
                            path_parameters: vec![("string".to_string(), "name".to_string())],
                        },
                    ],
                    skip: false,
                    readers: vec![],
                    writers: vec![],
                    document: None,
                    path_parameters: vec![],
                }],
                skip: false,
                readers: vec![],
                writers: vec![],
                document: None,
                path_parameters: vec![],
            }],
        });

        assert_eq!(left, right)
    }
}
