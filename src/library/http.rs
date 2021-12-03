pub fn processor(section: &ftd::p1::Section, doc: &ftd::p2::TDoc) -> ftd::p1::Result<ftd::Value> {
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
        if k == "$processor$" {
            continue;
        }
        url.query_pairs_mut().append_pair(k, v);
    }

    let json = futures::executor::block_on(get(url, doc.name, section.line_number))?;
    doc.from_json(&json, section)
}

async fn get(
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

    Ok(serde_json::from_str(t.as_str())?)
}

async fn _get(url: url::Url) -> reqwest::Result<String> {
    let c = reqwest::Client::builder().user_agent("fpm").build()?;
    c.get(url).send().await?.text().await
}
