pub fn document_full_id(
    req_config: &fastn_core::RequestConfig,
    doc: &ftd::interpreter::TDoc,
) -> ftd::ftd2021::p1::Result<String> {
    let full_document_id = req_config.doc_id().unwrap_or_else(|| {
        doc.name
            .to_string()
            .replace(req_config.config.package.name.as_str(), "")
    });

    if full_document_id.trim_matches('/').is_empty() {
        return Ok("/".to_string());
    }

    Ok(format!("/{}/", full_document_id.trim_matches('/')))
}

static GOOGLE_DATETIME_REGEX: once_cell::sync::Lazy<regex::Regex> =
    once_cell::sync::Lazy::new(|| {
        regex::Regex::new(r"Date\((\d+),(\d+),(\d+),?(\d+)?,?(\d+)?,?(\d+)?\)").unwrap()
    });

#[allow(dead_code)]
#[derive(Debug)]
pub(crate) struct ParsedDate {
    pub year: i32,
    pub month: u32,
    pub day: u32,
    pub hour: u32,
    pub minute: u32,
    pub second: u32,
    pub timestamp: i64,
}

impl ParsedDate {
    pub(crate) fn from_str(date_str: &str) -> Option<Self> {
        if let Some(captures) = GOOGLE_DATETIME_REGEX.captures(date_str) {
            let year: i32 = captures[1].parse().unwrap();
            let month: u32 = captures[2].parse().unwrap();
            let day: u32 = captures[3].parse().unwrap();
            let hour: u32 = captures
                .get(4)
                .map_or(0, |m| m.as_str().parse().unwrap_or(0));
            let minute: u32 = captures
                .get(5)
                .map_or(0, |m| m.as_str().parse().unwrap_or(0));
            let second: u32 = captures
                .get(6)
                .map_or(0, |m| m.as_str().parse().unwrap_or(0));

            let naive_date_time = chrono::NaiveDate::from_ymd_opt(year, month, day)?
                .and_hms_opt(hour, minute, second)?;
            let utc_date_time = chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(
                naive_date_time,
                chrono::Utc,
            );

            let timestamp = utc_date_time.timestamp();

            Some(ParsedDate {
                year,
                month,
                day,
                hour,
                minute,
                second,
                timestamp,
            })
        } else {
            None
        }
    }
}
