#[derive(Debug, serde::Deserialize, Clone)]
pub struct RedirectsTemp {
    #[serde(rename = "redirects-body")]
    pub body: String,
}

impl RedirectsTemp {
    pub(crate) fn redirects_from_body(&self) -> ftd::Map<String> {
        let body = self.body.as_str();
        let mut redirects: ftd::Map<String> = ftd::Map::new();
        for line in body.lines() {
            if line.trim_start().starts_with(';') {
                continue;
            }

            if let Some((key, value)) = line.split_once(':') {
                redirects.insert(key.trim().to_owned(), value.trim().to_owned());
            }
        }
        redirects
    }
}

pub fn find_redirect(redirects: &ftd::Map<String>, path: &str) -> Option<String> {
    let original = path;
    let fixed = format!(
        "/{}/",
        path.trim_matches('/')
            .trim_end_matches("index.ftd")
            .trim_end_matches(".ftd")
    );

    return if redirects.contains_key(original) {
        redirects.get(original).cloned()
    } else if redirects.contains_key(fixed.as_str()) {
        redirects.get(fixed.as_str()).cloned()
    } else {
        None
    };
}
