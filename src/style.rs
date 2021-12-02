#[derive(serde::Deserialize, Debug)]
pub struct Font {
    name: String,
    woff: Option<String>,
    woff2: Option<String>,
}

#[derive(serde::Deserialize, Debug)]
pub struct Style {
    pub fonts: Vec<Font>,
}

impl Font {
    pub fn parse(b: &ftd::p2::Document) -> Vec<Font> {
        b.to_owned().instances("fpm#font").unwrap()
    }

    pub fn to_html(&self) -> Option<String> {
        Some(format!(
            "
            @font-face {{
                font-family: {};
                src: url({});
            }}
            ",
            self.name,
            self.woff2.as_ref().unwrap_or_else(|| self
                .woff
                .as_ref()
                .expect("Either woff2 or woff 1 should be provided"))
        ))
    }
}

impl Style {
    pub fn to_html(&self) -> Option<String> {
        let generated_style = self.fonts.iter().fold("".to_string(), |c, f| {
            format!("{}\n{}", c, f.to_html().unwrap_or_else(|| "".to_string()))
        });
        return match generated_style.is_empty() {
            false => Some(format!("<style>{}</style>", generated_style)),
            _ => None,
        };
    }
}
