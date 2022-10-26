use crate::sitemap::section::{Section, Subsection};

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

        let dynamic_urls = DynamicUrls {
            sections: fpm::sitemap::construct_tree_util(parser.finalize()?),
        };

        dbg!(&dynamic_urls);

        if dynamic_urls.not_have_path_params() {
            return Err(fpm::sitemap::ParseError::InvalidDynamicUrls {
                message: "All the dynamic urls must contain dynamic params".to_string(),
            });
        }

        Ok(dynamic_urls)
    }

    // If any one does not have path parameters so return true
    pub fn not_have_path_params(&self) -> bool {
        fn check_toc(toc: &fpm::sitemap::toc::TocItem) -> bool {
            if toc.path_parameters.is_empty() {
                return true;
            }

            for toc in toc.children.iter() {
                if check_toc(toc) {
                    return true;
                }
            }
            false
        }

        fn check_sub_section(sub_section: &Subsection) -> bool {
            // Note: No need check subsection
            // if sub_section.path_parameters.is_empty() {
            //     return true;
            // }

            for toc in sub_section.toc.iter() {
                if check_toc(toc) {
                    return true;
                }
            }
            false
        }

        fn check_section(section: &Section) -> bool {
            // Note: No need check section
            // if section.path_parameters.is_empty() {
            //     return true;
            // }

            for sub_section in section.subsections.iter() {
                if check_sub_section(sub_section) {
                    return true;
                }
            }
            false
        }

        for section in self.sections.iter() {
            if check_section(section) {
                return true;
            }
        }
        false
    }

    pub fn resolve_document(&self, path: &str) -> fpm::Result<fpm::sitemap::ResolveDocOutput> {
        // fn resolve_in_toc(toc: &toc::TocItem, path: &str) -> fpm::Result<ResolveDocOutput> {
        //     if fpm::utils::ids_matches(toc.id.as_str(), path) {
        //         return Ok((toc.document.clone(), vec![]));
        //     }
        //
        //     for child in toc.children.iter() {
        //         let (document, path_prams) = resolve_in_toc(child, path)?;
        //         if document.is_some() {
        //             return Ok((document, path_prams));
        //         }
        //     }
        //     Ok((None, vec![]))
        // }
        //
        // fn resolve_in_sub_section(
        //     sub_section: &section::Subsection,
        //     path: &str,
        // ) -> fpm::Result<ResolveDocOutput> {
        //     if let Some(id) = sub_section.id.as_ref() {
        //         if fpm::utils::ids_matches(path, id.as_str()) {
        //             return Ok((sub_section.document.clone(), vec![]));
        //         }
        //     }
        //
        //     for toc in sub_section.toc.iter() {
        //         let (document, path_params) = resolve_in_toc(toc, path)?;
        //         if document.is_some() {
        //             return Ok((document, path_params));
        //         }
        //     }
        //
        //     Ok((None, vec![]))
        // }
        //
        // fn resolve_in_section(
        //     section: &section::Section,
        //     path: &str,
        // ) -> fpm::Result<ResolveDocOutput> {
        //     if fpm::utils::ids_matches(section.id.as_str(), path) {
        //         return Ok((section.document.clone(), vec![]));
        //     }
        //
        //     for subsection in section.subsections.iter() {
        //         let (document, path_params) = resolve_in_sub_section(subsection, path)?;
        //         if document.is_some() {
        //             return Ok((document, path_params));
        //         }
        //     }
        //     Ok((None, vec![]))
        // }
        //
        // for section in sitemap.sections.iter() {
        //     let (document, path_params) = resolve_in_section(section, path)?;
        //     if document.is_some() {
        //         return Ok((document, path_params));
        //     }
        // }
        Ok((None, vec![]))
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
