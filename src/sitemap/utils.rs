// # Input
// request_url: /abrark/foo/28/
// sitemap_url: /<string:username>/foo/<integer:age>/
// params_types: [(string, username), (integer, age)]
// # Output
// true

/*
enum PathParams {
    NamedParm{index: usize, param_name: String, param_type: String}
    Param{index: usize, value: String}
}
*/

pub fn url_match(
    request_url: &str,
    sitemap_params: &[(usize, String, Option<String>)],
) -> fpm::Result<(bool, Vec<(String, ftd::Value)>)> {
    use itertools::Itertools;
    // request_attrs: [abrark, foo, 28]
    let request_parts = request_url.trim_matches('/').split('/').collect_vec();
    // This should go to config request [username: abrark, age: 28]
    if request_parts.len().ne(&sitemap_params.len()) {
        return Ok((false, vec![]));
    }

    // match logic
    // req: [a, ak, foo]
    // d-urls: [(0, a, None), (1, username, Some(string)), (2, foo, None)]
    // [(param_name, value)]
    let mut path_parameters: Vec<(String, ftd::Value)> = vec![];
    let mut index = 0;
    for req_part in request_parts {
        let (_idx, value_or_param_name, value_type) = &sitemap_params[index];
        index += 1; // Note: Use it carefully next
        if value_type.is_none() {
            if req_part.eq(value_or_param_name) {
                continue;
            } else {
                return Ok((false, vec![]));
            }
        } else {
            if let Ok(value) = get_value_type(req_part, value_type.as_ref().unwrap()) {
                path_parameters.push((value_or_param_name.to_string(), value));
            } else {
                return Ok((false, vec![]));
            }
        }
    }
    return Ok((true, path_parameters));

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

/// Please check test case: `parse_path_params_test_0`
/// This method is for parsing the dynamic params from fpm.dynamic-urls
pub fn parse_path_params(
    url: &str,
) -> Result<Vec<(usize, String, Option<String>)>, fpm::sitemap::ParseError> {
    let mut output = vec![];
    let url = url.trim().trim_matches('/');

    // b/<string:username>/<integer:age>/foo
    let parts: Vec<&str> = url.split('/').collect();
    // parts: [b, <string:username>, <integer:age>, foo]
    let mut index = 0;
    for part in parts.into_iter().map(|x| x.trim()) {
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
        let output = super::parse_path_params("/b/<string:username>/<integer:age>/foo/");
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
        let output = super::parse_path_params("/b/ <  string  :  username > / <integer:age>/foo/");
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
        let output = super::parse_path_params("/b/ <  :  username > / <integer:age>/foo/");
        // let test_output = vec![
        //     (0, "b".to_string(), None),
        //     (1, "username".to_string(), Some("string".to_string())),
        //     (2, "age".to_string(), Some("integer".to_string())),
        //     (3, "foo".to_string(), None),
        // ];
        assert!(output.is_err())
    }

    // cargo test --package fpm --lib sitemap::utils::tests::url_match -- --nocapture
    #[test]
    fn url_match() {
        // "/<string:username>/foo/<integer:age>/",
        let output = super::url_match(
            "/arpita/foo/28/",
            &[
                (0, "username".to_string(), Some("string".to_string())),
                (1, "foo".to_string(), None),
                (2, "age".to_string(), Some("integer".to_string())),
            ],
        );

        let output = output.unwrap();
        assert!(output.0);
        assert_eq!(
            output.1,
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

    // cargo test --package fpm --lib sitemap::utils::tests::url_match_2 -- --nocapture
    #[test]
    fn url_match_2() {
        // Input:
        // request_url: /arpita/foo/28/
        // sitemap_url: /<integer:username>/foo/<integer:age>/
        // Output: false
        // Reason: `arpita` can not be converted into `integer`
        let output = super::url_match(
            "/arpita/foo/28/",
            &[
                (0, "username".to_string(), Some("integer".to_string())),
                (1, "foo".to_string(), None),
                (2, "age".to_string(), Some("integer".to_string())),
            ],
        );

        assert!(!output.unwrap().0)
    }

    // cargo test --package fpm --lib sitemap::utils::tests::url_match_3
    #[test]
    fn url_match_3() {
        // Input:
        // request_url: /arpita/foo/
        // sitemap_url: /<string:username>/foo/<integer:age>/
        // Output: false
        // Reason: There is nothing to match in request_url after `foo`
        //         against with sitemap_url `<integer:age>`
        let output = super::url_match(
            "/arpita/foo/",
            &[
                (0, "username".to_string(), Some("integer".to_string())),
                (1, "foo".to_string(), None),
                (2, "age".to_string(), Some("integer".to_string())),
            ],
        );
        assert!(!output.unwrap().0)
    }

    // cargo test --package fpm --lib sitemap::utils::tests::url_match_4 -- --nocapture
    #[test]
    fn url_match_4() {
        // sitemap_url: /b/<string:username>/person/,
        let output = super::url_match(
            "/b/a/person/",
            &[
                (0, "b".to_string(), None),
                (1, "username".to_string(), Some("string".to_string())),
                (2, "person".to_string(), None),
            ],
        );
        let output = output.unwrap();
        assert!(output.0);
        assert_eq!(
            output.1,
            vec![(
                "username".to_string(),
                ftd::Value::String {
                    text: "a".to_string(),
                    source: TextSource::Default
                }
            )]
        )
    }

    // cargo test --package fpm --lib sitemap::utils::tests::url_match_4_1
    #[test]
    fn url_match_4_1() {
        // sitemap_url: /a/<string:username>/person/,
        let output = super::url_match(
            "/b/a/person/",
            &[
                (0, "a".to_string(), None),
                (1, "username".to_string(), Some("string".to_string())),
                (2, "person".to_string(), None),
            ],
        );
        assert!(!output.unwrap().0)
    }

    // cargo test --package fpm --lib sitemap::utils::tests::url_match_5 -- --nocapture
    #[test]
    fn url_match_5() {
        // sitemap_url: /a/<string:username>/person/<integer:age>
        let output = super::url_match(
            "/a/abrark/person/28/",
            &[
                (0, "a".to_string(), None),
                (1, "username".to_string(), Some("string".to_string())),
                (2, "person".to_string(), None),
                (3, "age".to_string(), Some("integer".to_string())),
            ],
        );
        let output = output.unwrap();
        assert!(output.0);
        assert_eq!(
            output.1,
            vec![
                (
                    "username".to_string(),
                    ftd::Value::String {
                        text: "abrark".to_string(),
                        source: TextSource::Default
                    }
                ),
                ("age".to_string(), ftd::Value::Integer { value: 28 })
            ]
        );
    }
}
