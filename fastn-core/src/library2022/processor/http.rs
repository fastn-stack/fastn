use ftd::interpreter::FunctionExt;
use ftd::interpreter::{PropertyValueExt, ValueExt};

#[tracing::instrument(name = "http_processor", skip_all)]
pub async fn process(
    value: ftd_ast::VariableValue,
    kind: fastn_resolved::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &mut fastn_core::RequestConfig,
) -> ftd::interpreter::Result<fastn_resolved::Value> {
    // we can in future do a more fine-grained analysis if the response
    // is cacheable or not, say depending on HTTP Vary header, etc.
    req_config.response_is_cacheable = false;

    let (headers, line_number) = if let Ok(val) = value.get_record(doc.name) {
        (val.2.to_owned(), val.5.to_owned())
    } else {
        (ftd_ast::HeaderValues::new(vec![]), value.line_number())
    };

    let method = headers
        .get_optional_string_by_key("method", doc.name, line_number)?
        .unwrap_or_else(|| "GET".to_string())
        .to_lowercase();

    if method.as_str().ne("get") && method.as_str().ne("post") {
        return ftd::interpreter::utils::e2(
            format!("only GET and POST methods are allowed, found: {method}"),
            doc.name,
            line_number,
        );
    }

    let url = match headers.get_optional_string_by_key("url", doc.name, line_number)? {
        Some(v) if v.starts_with('$') => {
            if let Some((function_name, args)) = parse_function_call(&v) {
                let function_call = create_function_call(&function_name, args, doc, line_number)?;
                let function = doc.get_function(&function_name, line_number)?;

                if function.js.is_some() {
                    return ftd::interpreter::utils::e2(
                        format!(
                            "{v} is a function with JavaScript which is not supported for HTTP URLs"
                        ),
                        doc.name,
                        line_number,
                    );
                }

                match function.resolve(
                    &function_call.kind,
                    &function_call.values,
                    doc,
                    line_number,
                )? {
                    Some(resolved_value) => resolved_value.string(doc.name, line_number)?,
                    None => {
                        return ftd::interpreter::utils::e2(
                            format!("{v} function returned no value"),
                            doc.name,
                            line_number,
                        );
                    }
                }
            } else {
                // This is a simple variable reference - handle as before
                match doc.get_thing(v.as_str(), line_number) {
                    Ok(ftd::interpreter::Thing::Variable(var)) => var
                        .value
                        .resolve(doc, line_number)?
                        .string(doc.name, line_number)?,
                    Ok(v2) => {
                        return ftd::interpreter::utils::e2(
                            format!("{v} is not a variable or function, it's a {v2:?}"),
                            doc.name,
                            line_number,
                        );
                    }
                    Err(e) => {
                        return ftd::interpreter::utils::e2(
                            format!("${v} not found in the document: {e:?}"),
                            doc.name,
                            line_number,
                        );
                    }
                }
            }
        }
        Some(v) => v,
        None => {
            return ftd::interpreter::utils::e2(
                format!(
                    "'url' key is required when using `{}: http`",
                    ftd::PROCESSOR_MARKER
                ),
                doc.name,
                line_number,
            );
        }
    };

    let package = if let Some(package) = req_config.config.all_packages.get(doc.name) {
        package.clone()
    } else if let Some((doc_name, _)) = doc.name.split_once("/-/") {
        req_config
            .config
            .all_packages
            .get(doc_name)
            .map(|v| v.clone())
            .unwrap_or_else(|| req_config.config.package.clone())
    } else {
        req_config.config.package.clone()
    };

    let (mut url, mountpoint, mut conf) = {
        let (mut url, mountpoint, conf) =
            fastn_core::config::utils::get_clean_url(&package, req_config, url.as_str())
                .await
                .map_err(|e| ftd::interpreter::Error::ParseError {
                    message: format!("invalid url: {e:?}"),
                    doc_id: doc.name.to_string(),
                    line_number,
                })?;
        (url, mountpoint, conf)
    };

    tracing::info!(?url);

    let mut body = serde_json::Map::new();
    for header in headers.0 {
        if header.key.as_str() == ftd::PROCESSOR_MARKER
            || header.key.as_str() == "url"
            || header.key.as_str() == "method"
        {
            tracing::info!("Skipping header: {}", header.key);
            continue;
        }

        let value = header.value.string(doc.name)?;

        tracing::info!("Processing header: {}: {:?}", header.key, value);

        if let Some(key) = fastn_core::http::get_header_key(header.key.as_str()) {
            let value = value.to_string();
            tracing::info!("Adding header: {}: {}", key, value);
            conf.insert(key.to_string(), value);
            continue;
        }

        // 1 id: $query.id
        // After resolve headers: id:1234(value of $query.id)
        if value.starts_with('$') {
            tracing::info!("Resolving variable in header: {}", value);

            if let Some(value) = doc
                .get_value(header.line_number, value)?
                .to_serde_value(doc)?
            {
                tracing::info!("Resolved variable in header: {}: {:?}", header.key, value);
                if method.as_str().eq("post") {
                    body.insert(header.key, value);
                } else {
                    let value = match value {
                        serde_json::Value::String(s) => Some(s),
                        serde_json::Value::Null => None,
                        _ => Some(
                            serde_json::to_string(&value)
                                .map_err(|e| ftd::interpreter::Error::Serde { source: e })?,
                        ),
                    };

                    if let Some(value) = value {
                        url.query_pairs_mut()
                            .append_pair(header.key.as_str(), &value);
                    }
                }
            }
        } else {
            tracing::info!("Using static value in header: {}: {}", header.key, value);

            if method.as_str().eq("post") {
                body.insert(
                    header.key,
                    serde_json::Value::String(fastn_core::utils::escape_string(value)),
                );
            } else {
                // why not escape_string here?
                url.query_pairs_mut()
                    .append_pair(header.key.as_str(), value);
            }
        }
    }

    if !req_config.config.test_command_running {
        println!("calling `http` processor with url: {url}");
    }

    let resp = if url.scheme() == "wasm+proxy" {
        tracing::info!("Calling wasm+proxy with url: {url}");
        let mountpoint = mountpoint.ok_or(ftd::interpreter::Error::OtherError(
            "Mountpoint not found!".to_string(),
        ))?;
        let app_mounts = req_config
            .config
            .app_mounts()
            .map_err(|e| ftd::interpreter::Error::OtherError(e.to_string()))?;

        if method == "post" {
            req_config.request.body = serde_json::to_vec(&body)
                .map_err(|e| ftd::interpreter::Error::Serde { source: e })?
                .into();
        }

        match req_config
            .config
            .ds
            .handle_wasm(
                req_config.config.package.name.clone(),
                url.to_string(),
                &req_config.request,
                mountpoint,
                app_mounts,
                // FIXME: we don't know how to handle unsaved wasm files. Maybe there is no way
                // that an unsaved .wasm file can exist and this is fine.
                &None,
            )
            .await
        {
            Ok(r) => {
                match r.method.parse() {
                    Ok(200) => (),
                    Ok(code) => {
                        req_config.processor_set_response = Some(r);
                        // We return an error here, this error will not be shown to user,  but is created to to
                        // short-circuit further processing. the `.processor_set_response` will be handled by
                        // `fastn_core::commands::serve::shared_to_http`
                        return ftd::interpreter::utils::e2(
                            format!("wasm code returned non 200 {code}"),
                            doc.name,
                            line_number,
                        );
                    }
                    Err(e) => {
                        return ftd::interpreter::utils::e2(
                            format!("wasm code is not an integer {}: {e:?}", r.method.as_str()),
                            doc.name,
                            line_number,
                        );
                    }
                };

                let mut resp_cookies = vec![];
                r.headers.into_iter().for_each(|(k, v)| {
                    if k.as_str().eq("set-cookie")
                        && let Ok(v) = String::from_utf8(v)
                    {
                        resp_cookies.push(v.to_string());
                    }
                });
                Ok((Ok(r.body.into()), resp_cookies))
            }
            e => todo!("error: {e:?}"),
        }
    } else if method.as_str().eq("post") {
        tracing::info!("Calling POST request with url: {url}");
        fastn_core::http::http_post_with_cookie(
            req_config,
            url.as_str(),
            &conf,
            &serde_json::to_string(&body)
                .map_err(|e| ftd::interpreter::Error::Serde { source: e })?,
        )
        .await
        .map_err(|e| ftd::interpreter::Error::DSHttpError {
            message: format!("{e:?}"),
        })
    } else {
        tracing::info!("Calling GET request with url: {url}");
        fastn_core::http::http_get_with_cookie(
            &req_config.config.ds,
            &req_config.request,
            url.as_str(),
            &conf,
            false, // disable cache
        )
        .await
        .map_err(|e| ftd::interpreter::Error::DSHttpError {
            message: format!("{e:?}"),
        })
    };

    let response = match resp {
        Ok((Ok(v), cookies)) => {
            req_config.processor_set_cookies.extend(cookies);
            v
        }
        Ok((Err(e), cookies)) => {
            req_config.processor_set_cookies.extend(cookies);
            return ftd::interpreter::utils::e2(
                format!("HTTP::get failed: {e:?}"),
                doc.name,
                line_number,
            );
        }
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("HTTP::get failed: {e:?}"),
                doc.name,
                line_number,
            );
        }
    };

    let response_json: serde_json::Value = serde_json::from_slice(&response)
        .map_err(|e| ftd::interpreter::Error::Serde { source: e })?;

    doc.from_json(&response_json, &kind, &value)
}

/// Parse a function call string like "$function_name(arg1 = value1, arg2 = value2)" into name and named arguments
fn parse_function_call(expr: &str) -> Option<(String, Vec<(String, String)>)> {
    if !expr.starts_with('$') {
        return None;
    }

    let expr = &expr[1..];

    if let Some(open_paren) = expr.find('(')
        && let Some(close_paren) = expr.rfind(')')
        && close_paren > open_paren
    {
        let function_name = expr[..open_paren].trim().to_string();
        let args_str = &expr[open_paren + 1..close_paren];

        if args_str.trim().is_empty() {
            return Some((function_name, vec![]));
        }

        let mut args = vec![];

        for arg_pair in args_str.split(',') {
            let arg_pair = arg_pair.trim();
            if arg_pair.is_empty() {
                continue;
            }

            if let Some(eq_pos) = arg_pair.find('=') {
                let arg_name = arg_pair[..eq_pos].trim().to_string();
                let arg_value = arg_pair[eq_pos + 1..].trim().to_string();
                args.push((arg_name, arg_value));
            }
        }

        return Some((function_name, args));
    }

    None
}

/// Create a FunctionCall with the given named arguments
fn create_function_call(
    function_name: &str,
    args: Vec<(String, String)>,
    doc: &ftd::interpreter::TDoc,
    line_number: usize,
) -> ftd::interpreter::Result<fastn_resolved::FunctionCall> {
    let function = doc.get_function(function_name, line_number)?;

    let mut values = ftd::Map::new();
    let mut order = vec![];

    for (arg_name, arg_value) in args {
        let param = function
            .arguments
            .iter()
            .find(|p| p.name == arg_name)
            .ok_or_else(|| ftd::interpreter::Error::ParseError {
                message: format!("Function {function_name} has no parameter named '{arg_name}'"),
                doc_id: doc.name.to_string(),
                line_number,
            })?;

        let property_value = if let Some(name) = arg_value.strip_prefix('$') {
            fastn_resolved::PropertyValue::Reference {
                name: name.to_string(),
                // TODO(siddhantk232): type check function params and args. We don't have enough info here to
                // do this yet.
                kind: param.kind.clone(),
                // TODO: support component local args
                source: fastn_resolved::PropertyValueSource::Global,
                is_mutable: false,
                line_number,
            }
        } else {
            fastn_resolved::PropertyValue::Value {
                value: fastn_resolved::Value::String {
                    text: arg_value.clone(),
                },
                is_mutable: false,
                line_number,
            }
        };

        order.push(arg_name.clone());
        values.insert(arg_name, property_value);
    }

    Ok(fastn_resolved::FunctionCall {
        name: function_name.to_string(),
        kind: function.return_kind.clone(),
        is_mutable: false,
        line_number,
        values,
        order,
        module_name: None,
    })
}

#[cfg(test)]
mod tests {
    use super::parse_function_call;

    #[test]
    fn test_parse_function_call_simple() {
        let result = parse_function_call("$my-func()");
        assert_eq!(result, Some(("my-func".to_string(), vec![])));
    }

    #[test]
    fn test_parse_function_call_single_arg() {
        // Test function call with one argument
        let result = parse_function_call("$my-func(arg1 = value1)");
        assert_eq!(
            result,
            Some((
                "my-func".to_string(),
                vec![("arg1".to_string(), "value1".to_string())]
            ))
        );
    }

    #[test]
    fn test_parse_function_call_multiple_args() {
        let result = parse_function_call("$my-func(arg1 = value1, arg2 = value2)");
        assert_eq!(
            result,
            Some((
                "my-func".to_string(),
                vec![
                    ("arg1".to_string(), "value1".to_string()),
                    ("arg2".to_string(), "value2".to_string())
                ]
            ))
        );
    }

    #[test]
    fn test_parse_function_call_with_variables() {
        let result = parse_function_call("$my-func(arg1 = $some-var, arg2 = $another-var)");
        assert_eq!(
            result,
            Some((
                "my-func".to_string(),
                vec![
                    ("arg1".to_string(), "$some-var".to_string()),
                    ("arg2".to_string(), "$another-var".to_string())
                ]
            ))
        );
    }

    #[test]
    fn test_parse_function_call_mixed_args() {
        let result = parse_function_call("$my-func(literal = hello, variable = $some-var)");
        assert_eq!(
            result,
            Some((
                "my-func".to_string(),
                vec![
                    ("literal".to_string(), "hello".to_string()),
                    ("variable".to_string(), "$some-var".to_string())
                ]
            ))
        );
    }

    #[test]
    fn test_parse_function_call_hyphenated_names() {
        // Test function call with hyphenated function and argument names
        let result = parse_function_call("$some-func(arg-1 = val-1, arg-2 = val-2)");
        assert_eq!(
            result,
            Some((
                "some-func".to_string(),
                vec![
                    ("arg-1".to_string(), "val-1".to_string()),
                    ("arg-2".to_string(), "val-2".to_string())
                ]
            ))
        );
    }

    #[test]
    fn test_parse_function_call_with_spaces() {
        let result = parse_function_call("$my-func( arg1 = value1 , arg2 = value2 )");
        assert_eq!(
            result,
            Some((
                "my-func".to_string(),
                vec![
                    ("arg1".to_string(), "value1".to_string()),
                    ("arg2".to_string(), "value2".to_string())
                ]
            ))
        );
    }

    #[test]
    fn test_parse_function_call_complex_values() {
        let result =
            parse_function_call("$my-func(url = https://example.com/api, path = /some/path)");
        assert_eq!(
            result,
            Some((
                "my-func".to_string(),
                vec![
                    ("url".to_string(), "https://example.com/api".to_string()),
                    ("path".to_string(), "/some/path".to_string())
                ]
            ))
        );
    }

    #[test]
    fn test_parse_function_call_not_a_function() {
        assert_eq!(parse_function_call("$simple-var"), None);
        assert_eq!(parse_function_call("not-a-function"), None);
        assert_eq!(parse_function_call(""), None);
        assert_eq!(parse_function_call("$func-without-closing-paren("), None);
        assert_eq!(parse_function_call("$func-without-opening-paren)"), None);
    }

    #[test]
    // TODO: throw errors for these
    fn test_parse_function_call_malformed() {
        assert_eq!(
            parse_function_call("$func(arg without equals)"),
            Some(("func".to_string(), vec![]))
        );
        assert_eq!(
            parse_function_call("$func(arg1 = val1, malformed)"),
            Some((
                "func".to_string(),
                vec![("arg1".to_string(), "val1".to_string())]
            ))
        );
    }

    #[test]
    fn test_parse_function_call_empty_args() {
        let result = parse_function_call("$my-func(   )");
        assert_eq!(result, Some(("my-func".to_string(), vec![])));
    }

    #[test]
    fn test_parse_function_call_trailing_comma() {
        let result = parse_function_call("$my-func(arg1 = val1, arg2 = val2,)");
        assert_eq!(
            result,
            Some((
                "my-func".to_string(),
                vec![
                    ("arg1".to_string(), "val1".to_string()),
                    ("arg2".to_string(), "val2".to_string())
                ]
            ))
        );
    }
}
