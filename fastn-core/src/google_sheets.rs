const GOOGLE_SHEET_API_BASE_URL: &str = "https://docs.google.com/a/google.com/spreadsheets/d";

static GOOGLE_SHEETS_ID_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| regex::Regex::new(r"/spreadsheets/d/([a-zA-Z0-9-_]+)").unwrap());

static JSON_RESPONSE_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"^/\*O_o\*/\s*google.visualization.Query.setResponse\((.*?)\);$")
            .unwrap()
    });

pub(crate) fn extract_google_sheets_id(url: &str) -> Option<String> {
    if let Some(captures) = GOOGLE_SHEETS_ID_REGEX.captures(url) {
        if let Some(id) = captures.get(1) {
            return Some(id.as_str().to_string());
        }
    }

    None
}

pub(crate) fn extract_json(input: &str) -> ftd::interpreter::Result<Option<String>> {
    if let Some(captures) = JSON_RESPONSE_REGEX.captures(input) {
        match captures.get(1) {
            Some(m) => Ok(Some(m.as_str().to_string())),
            None => Ok(None),
        }
    } else {
        Ok(None)
    }
}

pub(crate) fn generate_google_sheet_url(google_sheet_id: &str) -> String {
    format!(
        "{}/{}/gviz/tq?tqx=out:json",
        GOOGLE_SHEET_API_BASE_URL, google_sheet_id,
    )
}

pub(crate) fn prepare_query_url(url: &str, query: &str, sheet: &Option<String>) -> String {
    let mut query_url = url::form_urlencoded::Serializer::new(url.to_string());

    query_url.append_pair("tq", query);

    if let Some(sheet) = sheet {
        query_url.append_pair("sheet", sheet);
    }

    query_url.finish()
}
