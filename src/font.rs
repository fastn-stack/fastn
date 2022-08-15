#[derive(serde::Deserialize, Debug, Clone)]
pub struct Font {
    pub name: String,
    woff: Option<String>,
    woff2: Option<String>,
    truetype: Option<String>,
    opentype: Option<String>,
    #[serde(rename = "embedded-opentype")]
    embedded_opentype: Option<String>,
    svg: Option<String>,
    #[serde(rename = "unicode-range")]
    unicode_range: Option<String>,
    display: Option<String>,
    style: Option<String>,
    weight: Option<String>,
    stretch: Option<String>,
}

pub(crate) fn escape(s: &str) -> String {
    let s = s.replace('>', "\\u003E");
    let s = s.replace('<', "\\u003C");
    s.replace('&', "\\u0026")
}

fn append_src(kind: &str, value: &Option<String>, collector: &mut Vec<String>) {
    if let Some(v) = value {
        collector.push(format!("url({}) format('{}')", escape(v), kind))
    }
}

impl Font {
    pub fn get_url(&self) -> Option<String> {
        if self.woff.is_some() {
            return self.woff.clone();
        }
        if self.woff2.is_some() {
            return self.woff2.clone();
        }
        if self.truetype.is_some() {
            return self.truetype.clone();
        }
        if self.opentype.is_some() {
            return self.opentype.clone();
        }
        if self.embedded_opentype.is_some() {
            return self.embedded_opentype.clone();
        }
        if self.svg.is_some() {
            return self.svg.clone();
        }
        None
    }

    pub fn to_html(&self, package_name: &str) -> String {
        let mut attrs = vec![];
        if let Some(ref ur) = self.unicode_range {
            attrs.push(format!("unicode-range: {}", escape(ur)));
        }
        if let Some(ref d) = self.display {
            attrs.push(format!("font-display: {}", escape(d)));
        }
        if let Some(ref d) = self.style {
            attrs.push(format!("font-style: {}", escape(d)));
        }
        if let Some(ref d) = self.weight {
            attrs.push(format!("font-weight: {}", escape(d)));
        }
        if let Some(ref d) = self.stretch {
            attrs.push(format!("font-stretch: {}", escape(d)));
        }

        let mut src: Vec<String> = vec![];
        append_src("woff", &self.woff, &mut src);
        append_src("woff2", &self.woff2, &mut src);
        append_src("truetype", &self.truetype, &mut src);
        append_src("opentype", &self.opentype, &mut src);
        append_src("embedded-opentype", &self.embedded_opentype, &mut src);
        append_src("svg", &self.svg, &mut src);

        if !src.is_empty() {
            attrs.push(format!("src: {}", src.join(", ")));
        }

        if attrs.is_empty() {
            "".to_string()
        } else {
            attrs.push(format!("font-family: {}", self.html_name(package_name)));
            format!("@font-face {{ {} }}", attrs.join(";\n"))
        }
    }

    pub fn html_name(&self, package_name: &str) -> String {
        // use sha2::Digest;
        let hash_str = format!("{}-{}", package_name, self.name.as_str());
        // let mut sha256 = sha2::Sha256::new();
        // sha256.update(hash_str);
        hash_str
            .chars()
            .map(|x| match x {
                '.' | '/' | '?' | '_' => '-',
                _ => x,
            })
            .collect()
    }
}
