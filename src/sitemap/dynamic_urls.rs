// document and path-parameters
pub(crate) type ResolveDocOutput = (Option<String>, Vec<(String, ftd::Value)>);

#[derive(Debug, serde::Deserialize, Clone)]
pub struct DynamicUrlsTemp {
    #[serde(rename = "dynamic-urls-body")]
    pub body: String,
}

#[derive(Debug, Clone, PartialEq)]
pub struct DynamicUrls {
    pub sections: Vec<fastn::sitemap::section::Section>,
}

impl DynamicUrls {
    pub fn parse(
        global_ids: &std::collections::HashMap<String, String>,
        package_name: &str,
        body: &str,
    ) -> Result<Self, fastn::sitemap::ParseError> {
        // Note: Using Sitemap Parser, because format of dynamic-urls is same as sitemap
        let mut parser = fastn::sitemap::SitemapParser {
            state: fastn::sitemap::ParsingState::WaitingForSection,
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
            sections: fastn::sitemap::construct_tree_util(parser.finalize()?),
        };

        if dynamic_urls.any_without_named_params() {
            return Err(fastn::sitemap::ParseError::InvalidDynamicUrls {
                message: "All the dynamic urls must contain dynamic params".to_string(),
            });
        }

        Ok(dynamic_urls)
    }

    // If any one does not have path parameters so return true
    // any_without_named_params
    pub fn any_without_named_params(&self) -> bool {
        fn any_named_params(v: &[fastn::sitemap::PathParams]) -> bool {
            v.iter().any(|x| x.is_named_param())
        }

        fn check_toc(toc: &fastn::sitemap::toc::TocItem) -> bool {
            if !any_named_params(&toc.path_parameters) {
                return true;
            }

            for toc in toc.children.iter() {
                if check_toc(toc) {
                    return true;
                }
            }
            false
        }

        fn check_sub_section(sub_section: &fastn::sitemap::section::Subsection) -> bool {
            // Note: No need to check subsection
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

        fn check_section(section: &fastn::sitemap::section::Section) -> bool {
            // Note: No need to check section
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

    #[tracing::instrument(name = "dynamic-urls-resolve-document")]
    pub fn resolve_document(&self, path: &str) -> fastn::Result<ResolveDocOutput> {
        fn resolve_in_toc(
            toc: &fastn::sitemap::toc::TocItem,
            path: &str,
        ) -> fastn::Result<ResolveDocOutput> {
            if !toc.path_parameters.is_empty() {
                // path: /arpita/foo/28/
                // request: arpita foo 28
                // sitemap: [string,integer]
                // Mapping: arpita -> string, foo -> foo, 28 -> integer
                let params =
                    fastn::sitemap::utils::url_match(path, toc.path_parameters.as_slice())?;

                if params.0 {
                    return Ok((toc.document.clone(), params.1));
                }
            }

            for child in toc.children.iter() {
                let (document, path_prams) = resolve_in_toc(child, path)?;
                if document.is_some() {
                    return Ok((document, path_prams));
                }
            }

            Ok((None, vec![]))
        }

        fn resolve_in_sub_section(
            sub_section: &fastn::sitemap::section::Subsection,
            path: &str,
        ) -> fastn::Result<ResolveDocOutput> {
            if !sub_section.path_parameters.is_empty() {
                // path: /arpita/foo/28/
                // request: arpita foo 28
                // sitemap: [string,integer]
                // Mapping: arpita -> string, foo -> foo, 28 -> integer
                let params =
                    fastn::sitemap::utils::url_match(path, sub_section.path_parameters.as_slice())?;

                if params.0 {
                    return Ok((sub_section.document.clone(), params.1));
                }
            }
            for toc in sub_section.toc.iter() {
                let (document, path_params) = resolve_in_toc(toc, path)?;
                if document.is_some() {
                    return Ok((document, path_params));
                }
            }

            Ok((None, vec![]))
        }

        fn resolve_in_section(
            section: &fastn::sitemap::section::Section,
            path: &str,
        ) -> fastn::Result<ResolveDocOutput> {
            // path: /abrark/foo/28/
            // In sitemap url: /<string:username>/foo/<integer:age>/
            if !section.path_parameters.is_empty() {
                // path: /abrark/foo/28/
                // request: abrark foo 28
                // sitemap: [string,integer]
                // params_matches: abrark -> string, foo -> foo, 28 -> integer
                let params =
                    fastn::sitemap::utils::url_match(path, section.path_parameters.as_slice())?;

                if params.0 {
                    return Ok((section.document.clone(), params.1));
                }
            }

            for subsection in section.subsections.iter() {
                let (document, path_params) = resolve_in_sub_section(subsection, path)?;
                if document.is_some() {
                    return Ok((document, path_params));
                }
            }
            Ok((None, vec![]))
        }

        for section in self.sections.iter() {
            let (document, path_params) = resolve_in_section(section, path)?;
            if document.is_some() {
                tracing::info!(msg = "return: document found", path = path);
                return Ok((document, path_params));
            }
        }

        tracing::info!(msg = "return: document not found", path = path);
        Ok((None, vec![]))
    }
}

#[cfg(test)]
mod tests {

    #[test]
    fn parse_dynamic_urls() {
        let left = fastn::sitemap::DynamicUrls::parse(
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

        let right = Ok(fastn::sitemap::DynamicUrls {
            sections: vec![fastn::sitemap::section::Section {
                id: "Dynamic Urls Section".to_string(),
                icon: None,
                bury: false,
                title: Some("Dynamic Urls Section".to_string()),
                file_location: None,
                translation_file_location: None,
                extra_data: Default::default(),
                is_active: false,
                nav_title: None,
                subsections: vec![fastn::sitemap::section::Subsection {
                    id: None,
                    icon: None,
                    bury: false,
                    title: None,
                    file_location: None,
                    translation_file_location: None,
                    visible: false,
                    extra_data: Default::default(),
                    is_active: false,
                    nav_title: None,
                    toc: vec![
                        fastn::sitemap::toc::TocItem {
                            id: "/person/<string:name>/".to_string(),
                            icon: None,
                            bury: false,
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
                            skip: false,
                            readers: vec!["readers/person".to_string()],
                            writers: vec!["writers/person".to_string()],
                            document: Some("person.ftd".to_string()),
                            confidential: true,
                            path_parameters: vec![
                                fastn::sitemap::PathParams::value(0, "person".to_string()),
                                fastn::sitemap::PathParams::named(
                                    1,
                                    "name".to_string(),
                                    "string".to_string(),
                                ),
                            ],
                        },
                        fastn::sitemap::toc::TocItem {
                            id: "/person/<string:name>/".to_string(),
                            icon: None,
                            bury: false,
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
                            skip: false,
                            readers: vec!["readers/person".to_string()],
                            writers: vec!["writers/person".to_string()],
                            document: Some("person.ftd".to_string()),
                            confidential: true,
                            path_parameters: vec![
                                fastn::sitemap::PathParams::value(0, "person".to_string()),
                                fastn::sitemap::PathParams::named(
                                    1,
                                    "name".to_string(),
                                    "string".to_string(),
                                ),
                            ],
                        },
                    ],
                    skip: false,
                    readers: vec![],
                    writers: vec![],
                    document: None,
                    confidential: true,
                    path_parameters: vec![],
                }],
                skip: false,
                confidential: true,
                readers: vec![],
                writers: vec![],
                document: None,
                path_parameters: vec![],
            }],
        });
        assert_eq!(left, right)
    }
}
