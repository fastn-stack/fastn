pub fn split_at(text: &str, at: &str) -> (String, Option<String>) {
    if let Some((p1, p2)) = text.split_once(at) {
        (p1.trim().to_string(), Some(p2.trim().to_string()))
    } else {
        (text.to_string(), None)
    }
}

#[cfg(test)]
pub const CAPTION: &str = ftd_p1::utils::CAPTION;
pub const BODY: &str = "$body$";
