#[derive(serde::Deserialize, Debug)]
pub struct Font {
    name: String,
    woff: Option<String>,
    woff2: Option<String>,
}

impl Font {
    pub fn to_html(&self) -> String {
        if let Some(v) = self.woff2.as_ref().or_else(|| self.woff.as_ref()) {
            format!(
                "
                @font-face {{
                    font-family: {};
                    src: url({});
                }}",
                self.name,
                v // TODO: escape() this or do URL validation
            )
        } else {
            "".to_string()
        }
    }
}
