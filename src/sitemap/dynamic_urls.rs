// document and path-parameters
type ResolveDocOutput = (Option<String>, Vec<(String, ftd::Value)>);

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

        fn check_sub_section(sub_section: &fpm::sitemap::section::Subsection) -> bool {
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

        fn check_section(section: &fpm::sitemap::section::Section) -> bool {
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

    pub fn resolve_document(&self, path: &str) -> fpm::Result<ResolveDocOutput> {
        fn resolve_in_toc(
            toc: &fpm::sitemap::toc::TocItem,
            path: &str,
        ) -> fpm::Result<ResolveDocOutput> {
            if !toc.path_parameters.is_empty() {
                // path: /arpita/foo/28/
                // request: arpita foo 28
                // sitemap: [string,integer]
                // Mapping: arpita -> string, foo -> foo, 28 -> integer
                let params = fpm::sitemap::utils::parse_named_params(
                    path,
                    toc.id.as_str(),
                    toc.path_parameters.as_slice(),
                );

                if params.is_ok() {
                    return Ok((toc.document.clone(), params?));
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
            sub_section: &fpm::sitemap::section::Subsection,
            path: &str,
        ) -> fpm::Result<ResolveDocOutput> {
            if !sub_section.path_parameters.is_empty() {
                // path: /arpita/foo/28/
                // request: arpita foo 28
                // sitemap: [string,integer]
                // Mapping: arpita -> string, foo -> foo, 28 -> integer
                if let Some(id) = sub_section.id.as_ref() {
                    let params = fpm::sitemap::utils::parse_named_params(
                        path,
                        id.as_str(),
                        sub_section.path_parameters.as_slice(),
                    );

                    if params.is_ok() {
                        return Ok((sub_section.document.clone(), params?));
                    }
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
            section: &fpm::sitemap::section::Section,
            path: &str,
        ) -> fpm::Result<ResolveDocOutput> {
            // path: /abrark/foo/28/
            // In sitemap url: /<string:username>/foo/<integer:age>/
            if !section.path_parameters.is_empty() {
                // path: /abrark/foo/28/
                // request: abrark foo 28
                // sitemap: [string,integer]
                // params_matches: abrark -> string, foo -> foo, 28 -> integer
                let params = fpm::sitemap::utils::parse_named_params(
                    path,
                    section.id.as_str(),
                    section.path_parameters.as_slice(),
                );

                if params.is_ok() {
                    return Ok((section.document.clone(), params?));
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
                return Ok((document, path_params));
            }
        }

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
                icon: None,
                bury: false,
                confidential: true,
                file_location: None,
                translation_file_location: None,
                extra_data: Default::default(),
                is_active: false,
                nav_title: None,
                subsections: vec![fpm::sitemap::section::Subsection {
                    id: None,
                    title: None,
                    icon: None,
                    bury: false,
                    file_location: None,
                    translation_file_location: None,
                    visible: false,
                    extra_data: Default::default(),
                    is_active: false,
                    nav_title: None,
                    confidential: true,
                    toc: vec![
                        fpm::sitemap::toc::TocItem {
                            id: "/person/<string:name>/".to_string(),
                            title: Some("Url 1".to_string()),
                            icon: None,
                            bury: false,
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
                            confidential: true,
                        },
                        fpm::sitemap::toc::TocItem {
                            id: "/person/<string:name>/".to_string(),
                            title: Some("Url 2".to_string()),
                            icon: None,
                            bury: false,
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
                            confidential: true,
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
