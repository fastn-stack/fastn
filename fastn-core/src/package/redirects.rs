#[derive(Debug, PartialEq)]
pub struct UrlMappings {
    pub redirects: ftd::Map<String>,
    pub endpoints: Vec<fastn_package::old_fastn::EndpointData>,
    // todo: add dynamic-urls
    // pub dynamic_urls: <some-type>
}

impl UrlMappings {
    pub fn new(
        redirects: ftd::Map<String>,
        endpoints: Vec<fastn_package::old_fastn::EndpointData>,
    ) -> UrlMappings {
        UrlMappings {
            redirects,
            endpoints,
        }
    }
}

#[derive(Debug, serde::Deserialize, Clone)]
pub struct UrlMappingsTemp {
    #[serde(rename = "url-mappings-body")]
    pub body: String,
}

impl UrlMappingsTemp {
    pub(crate) fn url_mappings_from_body(&self) -> fastn_core::Result<UrlMappings> {
        let url_mappings_body = self.body.as_str();
        self.find_redirects_and_endpoints(url_mappings_body)
    }

    fn find_redirects_and_endpoints(&self, body: &str) -> fastn_core::Result<UrlMappings> {
        let mut redirects: ftd::Map<String> = ftd::Map::new();
        let mut endpoints = vec![];
        for line in body.lines() {
            let line = line.trim();

            // Ignore comments and endpoints
            if line.is_empty() || line.starts_with(';') {
                continue;
            }

            // Supported Endpoint Syntax under fastn.url-mappings
            // /ftd/* -> http+proxy://fastn.com/ftd/*
            //
            // localhost+proxy - http://127.0.0.1
            // /docs/* -> localhost+proxy:<port>

            if line.contains("proxy") {
                if let Some((first, second)) = line.split_once("->") {
                    let mountpoint = first.trim().to_string();
                    let endpoint = second
                        .replace("http+proxy", "http")
                        .replace("localhost", "127.0.0.1")
                        .to_string();

                    assert!(mountpoint.ends_with('*'));
                    assert!(endpoint.ends_with('*'));

                    endpoints.push(fastn_package::old_fastn::EndpointData {
                        endpoint: endpoint.trim().trim_end_matches('*').to_string(),
                        mountpoint: mountpoint.trim().trim_end_matches('*').to_string(),
                        user_id: None,
                    });
                }
                continue;
            }

            // Supported Redirects Syntax under fastn.url-mappings
            // <some link>: <link to redirect>
            // <some link> -> <link to redirect>

            if let Some((key, value)) = line.split_once("->") {
                Self::assert_and_insert_redirect(key, value, &mut redirects)?;
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                fastn_core::warning!(
                    "Redirect syntax: '{key}: {value}' will be deprecated\nPlease use the '{key} \
                    -> {value}' redirect syntax instead."
                );
                Self::assert_and_insert_redirect(key, value, &mut redirects)?;
            }
        }
        Ok(UrlMappings::new(redirects, endpoints))
    }

    // Assert checks on redirects
    // - All redirects should be A -> B where A != B (Self loop)
    // - If A -> B exists then there can’t be A -> C where B != C
    //   (No duplicated values starting with the same A)
    fn assert_and_insert_redirect(
        from: &str,
        to: &str,
        redirects: &mut ftd::Map<String>,
    ) -> fastn_core::Result<()> {
        let from = from.trim().to_owned();
        let to = to.trim().to_owned();

        assert!(
            !from.eq(to.as_str()),
            "Redirect {} -> {} is invalid",
            from,
            to
        );
        assert!(
            !redirects.contains_key(from.as_str()),
            "Redirect {} -> {} is invalid, since {} -> {} already exists",
            from.as_str(),
            to.as_str(),
            from.as_str(),
            redirects.get(from.as_str()).unwrap(),
        );

        redirects.insert(from, to);
        Ok(())
    }
}

pub fn find_redirect<'a>(redirects: &'a ftd::Map<String>, path: &str) -> Option<&'a String> {
    let original = path;
    let fixed = format!(
        "/{}/",
        path.trim_matches('/')
            .trim_end_matches("index.ftd")
            .trim_end_matches(".ftd")
    );

    return if redirects.contains_key(original) {
        redirects.get(original)
    } else if redirects.contains_key(fixed.as_str()) {
        redirects.get(fixed.as_str())
    } else {
        None
    };
}

#[cfg(test)]
mod tests {
    #[test]
    fn url_mappings() {
        let body = format!(indoc::indoc! {"
                /blog/ -> /blogs/
                /ftd/* -> http+proxy://fastn.com/ftd/*
                /docs/ -> http://fastn.com/docs/
                /slides/* -> http+proxy://localhost:7999/*
        "});
        let url_mappings_temp = crate::package::redirects::UrlMappingsTemp { body };
        let url_mappings = url_mappings_temp.url_mappings_from_body().ok();

        let expected_endpoints = vec![
            fastn_package::old_fastn::EndpointData {
                endpoint: "http://fastn.com/ftd/".to_string(),
                mountpoint: "/ftd/".to_string(),
                user_id: None,
            },
            fastn_package::old_fastn::EndpointData {
                endpoint: "http://127.0.0.1:7999/".to_string(),
                mountpoint: "/slides/".to_string(),
                user_id: None,
            },
        ];

        let mut expected_redirects: ftd::Map<String> = ftd::Map::new();
        expected_redirects.extend([
            ("/blog/".to_string(), "/blogs/".to_string()),
            ("/docs/".to_string(), "http://fastn.com/docs/".to_string()),
        ]);

        assert!(url_mappings.is_some());
        let url_mappings = url_mappings.unwrap();

        assert_eq!(url_mappings.endpoints.clone(), expected_endpoints);
        assert_eq!(url_mappings.redirects.clone(), expected_redirects);
    }
}
