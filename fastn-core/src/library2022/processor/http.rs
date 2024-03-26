pub async fn process(
    value: ftd_ast::VariableValue,
    kind: ftd::interpreter::Kind,
    doc: &ftd::interpreter::TDoc<'_>,
    req_config: &mut fastn_core::RequestConfig,
) -> ftd::interpreter::Result<ftd::interpreter::Value> {
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
            format!("only GET and POST methods are allowed, found: {}", method),
            doc.name,
            line_number,
        );
    }

    let url = match headers.get_optional_string_by_key("url", doc.name, line_number)? {
        Some(v) if v.starts_with('$') => match doc.get_thing(v.as_str(), line_number) {
            Ok(ftd::interpreter::Thing::Variable(v)) => v
                .value
                .resolve(doc, line_number)?
                .string(doc.name, line_number)?,
            Ok(v2) => {
                return ftd::interpreter::utils::e2(
                    format!("{v} is not a variable, it's a {v2:?}"),
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
        },
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

    let (mut url, mut conf) =
        fastn_core::config::utils::get_clean_url(&req_config.config, url.as_str()).map_err(
            |e| ftd::interpreter::Error::ParseError {
                message: format!("invalid url: {:?}", e),
                doc_id: doc.name.to_string(),
                line_number,
            },
        )?;

    let mut body = vec![];
    for header in headers.0 {
        if header.key.as_str() == ftd::PROCESSOR_MARKER
            || header.key.as_str() == "url"
            || header.key.as_str() == "method"
        {
            continue;
        }

        let value = header.value.string(doc.name)?;

        // 1 id: $query.id
        // After resolve headers: id:1234(value of $query.id)
        if value.starts_with('$') {
            if let Some(value) = doc
                .get_value(header.line_number, value.as_str())?
                .to_json_string(doc, true)?
            {
                if let Some(key) = fastn_core::http::get_header_key(header.key.as_str()) {
                    conf.insert(key.to_string(), value.trim_matches('"').to_string());
                    continue;
                }
                if method.as_str().eq("post") {
                    body.push(format!("\"{}\": {}", header.key, value));
                    continue;
                }
                url.query_pairs_mut()
                    .append_pair(header.key.as_str(), value.trim_matches('"'));
            }
        } else {
            if let Some(key) = fastn_core::http::get_header_key(header.key.as_str()) {
                conf.insert(key.to_string(), value);
                continue;
            }
            if method.as_str().eq("post") {
                body.push(format!(
                    "\"{}\": \"{}\"",
                    header.key,
                    fastn_core::utils::escape_string(value.as_str())
                ));
                continue;
            }
            url.query_pairs_mut()
                .append_pair(header.key.as_str(), value.as_str());
        }
    }

    if !req_config.config.test_command_running {
        println!("calling `http` processor with url: {}", &url);
    }

    let resp = if method.as_str().eq("post") {
        fastn_core::http::http_post_with_cookie(
            req_config,
            url.as_str(),
            &conf,
            format!("{{{}}}", body.join(",")).as_str(),
        )
        .await
        .map_err(|e| ftd::interpreter::Error::DSHttpError {
            message: format!("{:?}", e),
        })
    } else {
        fastn_core::http::http_get_with_cookie(
            &req_config.config.ds,
            &req_config.request,
            url.as_str(),
            &conf,
            false, // disable cache
        )
        .await
        .map_err(|e| ftd::interpreter::Error::DSHttpError {
            message: format!("{:?}", e),
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
                format!("HTTP::get failed: {:?}", e),
                doc.name,
                line_number,
            );
        }
        Err(e) => {
            return ftd::interpreter::utils::e2(
                format!("HTTP::get failed: {:?}", e),
                doc.name,
                line_number,
            );
        }
    };

    let response_string =
        String::from_utf8(response.to_vec()).map_err(|e| ftd::interpreter::Error::ParseError {
            message: format!("`http` processor API response error: {}", e),
            doc_id: doc.name.to_string(),
            line_number,
        })?;
    let response_json: serde_json::Value = serde_json::from_str(&response_string)
        .map_err(|e| ftd::interpreter::Error::Serde { source: e })?;

    doc.from_json(&response_json, &kind, &value)
}
