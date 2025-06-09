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
            fastn_core::config::utils::get_clean_url(&package, req_config, url.as_str()).map_err(
                |e| ftd::interpreter::Error::ParseError {
                    message: format!("invalid url: {:?}", e),
                    doc_id: doc.name.to_string(),
                    line_number,
                },
            )?;
        if !req_config.request.query_string().is_empty() {
            url.set_query(Some(req_config.request.query_string()));
        }
        (url, mountpoint, conf)
    };

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

        // 1 id: $query.id
        // After resolve headers: id:1234(value of $query.id)
        if value.starts_with('$') {
            tracing::info!("Resolving variable in header: {}", value);

            if let Some(value) = doc
                .get_value(header.line_number, value)?
                .to_json_string(doc, true)?
            {
                if let Some(key) = fastn_core::http::get_header_key(header.key.as_str()) {
                    conf.insert(key.to_string(), value.trim_matches('"').to_string());
                    continue;
                }
                if method.as_str().eq("post") {
                    body.insert(header.key, serde_json::Value::String(value));
                    continue;
                }
                url.query_pairs_mut()
                    .append_pair(header.key.as_str(), value.trim_matches('"'));
            }
        } else {
            if let Some(key) = fastn_core::http::get_header_key(header.key.as_str()) {
                conf.insert(key.to_string(), value.to_string());
                continue;
            }
            if method.as_str().eq("post") {
                body.insert(
                    header.key,
                    serde_json::Value::String(fastn_core::utils::escape_string(value)),
                );
                continue;
            }
            url.query_pairs_mut()
                .append_pair(header.key.as_str(), value);
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
                    if k.as_str().eq("set-cookie") {
                        if let Ok(v) = String::from_utf8(v) {
                            resp_cookies.push(v.to_string());
                        }
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
            message: format!("{:?}", e),
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
