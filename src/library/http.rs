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
            return ftd::e2(
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
            return ftd::e2(
                "'url' key is required when using `$processor$: http`",
                doc.name,
                section.line_number,
            )
        }
    };

    let mut url = match url::Url::parse(url.as_str()) {
        Ok(v) => v,
        Err(e) => {
            return ftd::e2(
                format!("invalid url: {:?}", e),
                doc.name,
                section.line_number,
            )
        }
    };

    for (_, k, v) in section.header.0.iter() {
        if k == "$processor$" || k == "url" || k == "method" {
            continue;
        }
        url.query_pairs_mut().append_pair(k, v);
    }

    let json = get(url, doc.name, section.line_number).await?;
    doc.from_json(&json, section)
}

pub(crate) async fn get(
    url: url::Url,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<serde_json::Value> {
    let t = match _get(url).await {
        Ok(v) => v,
        Err(e) => {
            return ftd::e2(
                format!("failed to fetch data: {:?}", e),
                doc_id,
                line_number,
            )
        }
    };

    match serde_json::from_str(t.as_str()) {
        Ok(v) => Ok(v),
        Err(e) => {
            eprintln!("failed to parse JSON: {}", t);
            Err(e.into())
        }
    }
}

async fn _get(url: url::Url) -> reqwest::Result<String> {
    let mut headers = reqwest::header::HeaderMap::new();
    headers.insert(
        reqwest::header::USER_AGENT,
        reqwest::header::HeaderValue::from_static("fpm"),
    );
    let c = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;
    c.get(url.to_string().as_str()).send().await?.text().await
}

pub async fn get_with_type<T: serde::de::DeserializeOwned>(
    url: url::Url,
    headers: reqwest::header::HeaderMap,
) -> fpm::Result<T> {
    let c = reqwest::Client::builder()
        .default_headers(headers)
        .build()?;

    let resp = c.get(url.to_string().as_str()).send().await?;
    if !resp.status().eq(&reqwest::StatusCode::OK) {
        return Err(fpm::Error::APIResponseError(format!(
            "url: {}, response_status: {}, response: {:?}",
            url,
            resp.status(),
            resp.text().await
        )));
    }
    Ok(resp.json().await?)
}
