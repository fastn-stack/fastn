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