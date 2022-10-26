// # Input
// request_url: /arpita/foo/28/
// sitemap_url: /<string:username>/foo/<integer:age>/
// params_types: [(string, username), (integer, age)]
// # Output
// true

pub fn parse_named_params(
    request_url: &str,
    sitemap_url: &str,
    params_type: &[(String, String)],
) -> fpm::Result<Vec<(String, ftd::Value)>> {
    use itertools::Itertools;
    // request_attrs: [arpita, foo, 28]
    let request_attrs = request_url.trim_matches('/').split('/').collect_vec();
    // sitemap_attrs: [<string:username>, foo, <integer:age>]
    let sitemap_attrs = sitemap_url.trim_matches('/').split('/').collect_vec();

    // This should go to config request [username: arpita, age: 28]

    if request_attrs.len().ne(&sitemap_attrs.len()) {
        return Err(fpm::Error::GenericError("".to_string()));
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
            dbg!(&attribute_value, attribute_type);
            value_parse_to_type(attribute_value, attribute_type)
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

    fn value_parse_to_type(value: &str, r#type: &str) -> fpm::Result<ftd::Value> {
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

// url: /<string:username>/<integer:age>/ => [("string", "username"), ("integer", "age")]
pub fn parse_path_params(url: &str) -> Vec<(String, String)> {
    fn path_params_regex() -> &'static regex::Regex {
        static PP: once_cell::sync::OnceCell<regex::Regex> = once_cell::sync::OnceCell::new();
        PP.get_or_init(|| {
            regex::Regex::new(r"<\s*([a-z]\w+)\s*:\s*([a-z|A-Z|0-9|_]\w+)\s*>")
                .expect("PATH_PARAMS: Regex is wrong")
        })
    }

    path_params_regex()
        .captures_iter(url)
        .into_iter()
        .map(|params| (params[1].to_string(), params[2].to_string()))
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests {
    use ftd::TextSource;

    #[test]
    fn parse_path_params_test_1() {
        let output = super::parse_path_params("/<string:username>/<integer:age>/");
        let test_output = vec![
            ("string".to_string(), "username".to_string()),
            ("integer".to_string(), "age".to_string()),
        ];
        assert_eq!(test_output, output)
    }

    #[test]
    fn parse_path_params_test_2() {
        let output = super::parse_path_params("/< string: username >/< integer: age >/");
        let test_output = vec![
            ("string".to_string(), "username".to_string()),
            ("integer".to_string(), "age".to_string()),
        ];
        assert_eq!(test_output, output)
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
}
