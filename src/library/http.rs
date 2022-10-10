pub async fn processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
) -> ftd::p1::Result<ftd::Value> {
    {
        let method = section
            .header
            .str_with_default(doc.name, section.line_number, "method", "GET")?
            .to_lowercase();

        if method != "get" {
            return ftd::p2::utils::e2(
                format!("only GET method is allowed, found: {}", method),
                doc.name,
                section.line_number,
            );
        }
    }

    let url = match section
        .header
        .string_optional(doc.name, section.line_number, "url")?
    {
        Some(v) => v,
        None => {
            return ftd::p2::utils::e2(
                "'url' key is required when using `$processor$: http`",
                doc.name,
                section.line_number,
            )
        }
    };

    let mut url = match url::Url::parse(url.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return ftd::p2::utils::e2(
                format!("invalid url: {:?}", e),
                doc.name,
                section.line_number,
            )
        }
    };

    for (line, key, value) in section.header.0.iter() {
        if key == "$processor$" || key == "url" || key == "method" {
            continue;
        }

        // 1 id: $query.id
        // After resolve headers: id:1234(value of $query.id)
        if value.starts_with('$') {
            if let Some(value) = doc.get_value(*line, value)?.to_string() {
                url.query_pairs_mut().append_pair(key, &value);
            }
        } else {
            url.query_pairs_mut().append_pair(key, value);
        }
    }

    println!("calling `http` processor with url: {}", &url);

    let response = match crate::http::http_get(url.as_str()).await {
        Ok(v) => v,
        Err(e) => {
            return ftd::p2::utils::e2(
                format!("HTTP::get failed: {:?}", e),
                doc.name,
                section.line_number,
            )
        }
    };

    let response_string = String::from_utf8(response).map_err(|e| ftd::p1::Error::ParseError {
        message: format!("`http` processor API response error: {}", e),
        doc_id: doc.name.to_string(),
        line_number: section.line_number,
    })?;
    let response_json: serde_json::Value =
        serde_json::from_str(&response_string).map_err(|e| ftd::p1::Error::Serde { source: e })?;

    doc.from_json(&response_json, section)
}

// Need to pass the request object also
// From request get the url, get query parameters, get the data from body(form data, post data)
pub fn request_data_processor<'a>(
    section: &ftd::p1::Section,
    doc: &ftd::p2::TDoc<'a>,
    config: &fpm::Config,
) -> ftd::p1::Result<ftd::Value> {
    // TODO: URL params not yet handled
    let req = match config.request.as_ref() {
        Some(v) => v,
        None => {
            return ftd::p2::utils::e2(
                "HttpRequest object should not be null",
                doc.name,
                section.line_number,
            )
        }
    };
    let mut data = req.query().clone();

    match req.body_as_json() {
        Ok(Some(b)) => {
            data.extend(b);
        }
        Ok(None) => {}
        Err(e) => {
            return ftd::p2::utils::e2(
                format!("Error while parsing request body: {:?}", e),
                doc.name,
                section.line_number,
            )
        }
    }

    doc.from_json(&data, section)
}
