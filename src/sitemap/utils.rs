// # Input
// request_url: /abrark/foo/28/
// sitemap_url: /<string:username>/foo/<integer:age>/
// params_types: [(string, username), (integer, age)]
// # Output
// true
// TODO: This should be match method
pub fn parse_named_params(
    request_url: &str,
    sitemap_url: &str,
    params_type: &[(usize, String, Option<String>)],
) -> fpm::Result<Vec<(String, ftd::Value)>> {
    use itertools::Itertools;
    // request_attrs: [abrark, foo, 28]
    let request_attrs = request_url.trim_matches('/').split('/').collect_vec();
    // sitemap_attrs: [<string:username>, foo, <integer:age>]
    let sitemap_attrs = sitemap_url.trim_matches('/').split('/').collect_vec();

    // This should go to config request [username: abrark, age: 28]
    if request_attrs.len().ne(&sitemap_attrs.len()) {
        return Err(fpm::Error::GenericError(
            "request attributes and sitemap attributes are not same".to_string(),
        ));
    }

    // [(param_name, value)]
    let mut path_parameters: Vec<(String, ftd::Value)> = vec![];

    // For every element either value should match or request attribute type should match to
    // sitemap's params_types
    let mut type_matches_count = 0;
    for idx in 0..request_attrs.len() {
        // either value match or type match
        let value_match = request_attrs[idx].eq(sitemap_attrs[idx]);
        if value_match {
            continue;
        }

        let parsed_value = {
            // request's attribute value type == type stored in sitemap:params_type
            let attribute_value = request_attrs[idx];
            assert!(params_type.len() > type_matches_count);
            let attribute_type = &params_type[type_matches_count].0;
            get_value_type(attribute_value, attribute_type)
        };
        match parsed_value {
            Ok(value) => {
                let attribute_name = params_type[type_matches_count].1.to_string();
                path_parameters.push((attribute_name, value));
            }
            Err(e) => return Err(fpm::Error::GenericError(e.to_string())),
        };

        type_matches_count += 1;
    }
    return Ok(path_parameters);

    fn get_value_type(value: &str, r#type: &str) -> fpm::Result<ftd::Value> {
        match r#type {
            "string" => Ok(ftd::Value::String {
                text: value.to_string(),
                source: ftd::TextSource::Default,
            }),
            "integer" => {
                let value = value.parse::<i64>()?;
                Ok(ftd::Value::Integer { value })
            }
            "decimal" => {
                let value = value.parse::<f64>()?;
                Ok(ftd::Value::Decimal { value })
            }
            "boolean" => {
                let value = value.parse::<bool>()?;
                Ok(ftd::Value::Boolean { value })
            }
            _ => unimplemented!(),
        }
    }
}

// Input:  /b/<string:username>/<integer:age>/foo/
// Output:vec![
//         (0, "b".to_string(), None),
//         (1, "username".to_string(), Some("string".to_string())),
//         (2, "age".to_string(), Some("integer".to_string())),
//         (3, "foo".to_string(), None),
//     ]
pub fn parse_path_params_new(
    url: &str,
) -> Result<Vec<(usize, String, Option<String>)>, fpm::sitemap::ParseError> {
    let mut output = vec![];
    let url = url.trim().trim_matches('/');

    // b/<string:username>/<integer:age>/foo
    let parts: Vec<&str> = url.split('/').collect();
    // parts: [b, <string:username>, <integer:age>, foo]
    let mut index = 0;
    for part in parts {
        let part = part.trim();
        if !part.is_empty() {
            if part.contains(':') && part.starts_with('<') && part.ends_with('>') {
                // <string:username>
                if let Some(colon_index) = part.find(':') {
                    let type_part = part[1..colon_index].trim();
                    let param_name_part = part[colon_index + 1..part.len() - 1].trim();
                    if type_part.is_empty() || param_name_part.is_empty() {
                        return Err(fpm::sitemap::ParseError::InvalidDynamicUrls {
                            message: format!("dynamic-urls format is wrong for: {}", part),
                        });
                    }
                    output.push((
                        index,
                        param_name_part.to_string(),
                        Some(type_part.to_string()),
                    ));
                    index += 1;
                }
            } else {
                // b
                output.push((index, part.to_string(), None));
                index += 1;
            }
        }
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use ftd::TextSource;

    // cargo test --package fpm --lib sitemap::utils::tests::parse_path_params_test_0
    #[test]
    fn parse_path_params_test_0() {
        let output = super::parse_path_params_new("/b/<string:username>/<integer:age>/foo/");
        let test_output = vec![
            (0, "b".to_string(), None),
            (1, "username".to_string(), Some("string".to_string())),
            (2, "age".to_string(), Some("integer".to_string())),
            (3, "foo".to_string(), None),
        ];
        assert!(output.is_ok());
        assert_eq!(test_output, output.unwrap())
    }

    // cargo test --package fpm --lib sitemap::utils::tests::parse_path_params_test_01
    #[test]
    fn parse_path_params_test_01() {
        let output =
            super::parse_path_params_new("/b/ <  string  :  username > / <integer:age>/foo/");
        let test_output = vec![
            (0, "b".to_string(), None),
            (1, "username".to_string(), Some("string".to_string())),
            (2, "age".to_string(), Some("integer".to_string())),
            (3, "foo".to_string(), None),
        ];
        assert!(output.is_ok());
        assert_eq!(test_output, output.unwrap())
    }

    // cargo test --package fpm --lib sitemap::utils::tests::parse_path_params_test_01
    #[test]
    fn parse_path_params_test_02() {
        let output = super::parse_path_params_new("/b/ <  :  username > / <integer:age>/foo/");
        // let test_output = vec![
        //     (0, "b".to_string(), None),
        //     (1, "username".to_string(), Some("string".to_string())),
        //     (2, "age".to_string(), Some("integer".to_string())),
        //     (3, "foo".to_string(), None),
        // ];
        assert!(output.is_err())
    }

    #[test]
    fn parse_named_params() {
        let output = super::parse_named_params(
            "/arpita/foo/28/",
            "/<string:username>/foo/<integer:age>/",
            &[
                ("string".to_string(), "username".to_string()),
                ("integer".to_string(), "age".to_string()),
            ],
        );

        assert_eq!(
            output.unwrap(),
            vec![
                (
                    "username".to_string(),
                    ftd::Value::String {
                        text: "arpita".to_string(),
                        source: TextSource::Default
                    }
                ),
                ("age".to_string(), ftd::Value::Integer { value: 28 })
            ]
        )
    }

    #[test]
    fn parse_named_params_1() {
        // Input:
        // request_url: /arpita/foo/28/
        // sitemap_url: /<string:username>/foo/<integer:age>/
        // params_types: [(string, username), (integer, age)]
        // Output: true
        // Reason: Everything is matching

        let output = super::parse_named_params(
            "/arpita/foo/28/",
            "/<string:username>/foo/<integer:age>/",
            &[
                ("string".to_string(), "username".to_string()),
                ("integer".to_string(), "age".to_string()),
            ],
        );

        assert!(output.is_ok())
    }

    #[test]
    fn parse_named_params_2() {
        // Input:
        // request_url: /arpita/foo/28/
        // sitemap_url: /<integer:username>/foo/<integer:age>/
        // params_types: [(integer, username), (integer, age)]
        // Output: false
        // Reason: `arpita` can not be converted into `integer`
        let output = super::parse_named_params(
            "/arpita/foo/28/",
            "/<integer:username>/foo/<integer:age>/",
            &[
                ("integer".to_string(), "username".to_string()),
                ("integer".to_string(), "age".to_string()),
            ],
        );

        assert!(output.is_err())
    }

    #[test]
    fn parse_named_params_3() {
        // Input:
        // request_url: /arpita/foo/
        // sitemap_url: /<string:username>/foo/<integer:age>/
        // params_types: [(string, username), (integer, age)]
        // Output: false
        // Reason: There is nothing to match in request_url after `foo`
        //         against with sitemap_url `<integer:age>`
        let output = super::parse_named_params(
            "/arpita/foo/",
            "/<string:username>/foo/<integer:age>/",
            &[
                ("string".to_string(), "username".to_string()),
                ("integer".to_string(), "age".to_string()),
            ],
        );

        assert!(output.is_err())
    }

    // cargo test --package fpm --lib sitemap::utils::tests::parse_named_params_4
    #[test]
    fn parse_named_params_4() {
        let output = super::parse_named_params(
            "/b/a/person/",
            "/b/<string:username>/person/",
            &[("string".to_string(), "username".to_string())],
        );
        assert!(output.is_ok())
    }

    // cargo test --package fpm --lib sitemap::utils::tests::parse_named_params_4_1
    #[test]
    fn parse_named_params_4_1() {
        let output = super::parse_named_params(
            "/b/a/person/",
            "/a/<string:username>/person/",
            &[("string".to_string(), "username".to_string())],
        );
        assert!(output.is_ok())
    }

    // cargo test --package fpm --lib sitemap::utils::tests::parse_named_params_5
    #[test]
    fn parse_named_params_5() {
        let output = super::parse_named_params(
            "/a/abrark/person/28/",
            "/a/<string:username>/person/<integer:age>",
            &[
                ("string".to_string(), "username".to_string()),
                ("integer".to_string(), "age".to_string()),
            ],
        );
        assert!(output.is_ok())
    }
}
