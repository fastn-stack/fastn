#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub struct Section {
    pub name: String,
    pub kind: Option<String>,
    pub caption: Option<ftd::p11::Header>,
    pub headers: Vec<ftd::p11::Header>,
    pub body: Option<Body>,
    pub sub_sections: Vec<Section>,
    pub is_commented: bool,
    pub line_number: usize,
    pub block_body: bool,
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct Body {
    pub line_number: usize,
    pub value: String,
}

impl Body {
    pub(crate) fn body(line_number: usize, value: &str) -> Body {
        Body {
            line_number,
            value: value.to_string(),
        }
    }
}
