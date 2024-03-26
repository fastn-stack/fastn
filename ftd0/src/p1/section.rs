use itertools::Itertools;

/**
 * Structure representing a section in a document.
 *
 * # Fields
 *
 * - `name`: A String representing the name of the section
 * - `kind`: An optional String representing the kind of the section
 * - `caption`: An optional `ftd0::p1::Header` representing the caption of the section
 * - `headers`: `ftd0::p1::Headers` representing the headers of the section
 * - `body`: An optional `Body` representing the body of the section
 * - `sub_sections`: A Vec of `Section` representing the sub sections of the section
 * - `is_commented`: A boolean representing whether the section is commented or not
 * - `line_number`: A usize representing the line number where the section starts in the document
 * - `block_body`: A boolean representing whether the section body is present as a block
 *
 */
#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize, Default)]
#[serde(default)]
pub struct Section {
    pub name: String,
    pub kind: Option<String>,
    pub caption: Option<ftd0::p1::Header>,
    pub headers: ftd0::p1::Headers,
    pub body: Option<Body>,
    pub sub_sections: Vec<Section>,
    pub is_commented: bool,
    pub line_number: usize,
    pub block_body: bool,
}

impl Section {
    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            kind: None,
            caption: None,
            body: None,
            sub_sections: vec![],
            is_commented: false,
            line_number: 0,
            headers: ftd0::p1::Headers(vec![]),
            block_body: false,
        }
    }

    pub fn add_sub_section(mut self, sub: Self) -> Self {
        self.sub_sections.push(sub);
        self
    }

    pub fn without_line_number(&self) -> Self {
        Self {
            name: self.name.to_string(),
            kind: self.kind.to_owned(),
            caption: self.caption.as_ref().map(|v| v.without_line_number()),
            headers: ftd0::p1::Headers(
                self.headers
                    .0
                    .iter()
                    .map(|v| v.without_line_number())
                    .collect_vec(),
            ),
            body: self.body.as_ref().map(|v| v.without_line_number()),
            sub_sections: self
                .sub_sections
                .iter()
                .map(|v| v.without_line_number())
                .collect_vec(),
            is_commented: self.is_commented.to_owned(),
            line_number: 0,
            block_body: false,
        }
    }

    pub fn and_caption(mut self, caption: &str) -> Self {
        self.caption = Some(ftd0::p1::Header::from_caption(caption, self.line_number));
        self
    }

    #[cfg(test)]
    pub(crate) fn list(self) -> Vec<Self> {
        vec![self]
    }

    pub fn add_header_str(mut self, key: &str, value: &str) -> Self {
        self.headers.push(ftd0::p1::Header::kv(
            0,
            key,
            None,
            if value.trim().is_empty() {
                None
            } else {
                Some(value.to_string())
            },
            None,
            Default::default(),
        ));
        self
    }

    pub fn add_header_str_with_source(
        mut self,
        key: &str,
        value: &str,
        source: Option<ftd0::p1::header::KVSource>,
    ) -> Self {
        self.headers.push(ftd0::p1::Header::kv(
            0,
            key,
            None,
            if value.trim().is_empty() {
                None
            } else {
                Some(value.to_string())
            },
            None,
            source,
        ));
        self
    }

    pub fn add_header_section(
        mut self,
        key: &str,
        kind: Option<String>,
        section: Vec<ftd0::p1::Section>,
        condition: Option<String>,
    ) -> Self {
        self.headers
            .push(ftd0::p1::Header::section(0, key, kind, section, condition));
        self
    }

    pub fn and_body(mut self, body: &str) -> Self {
        self.body = Some(ftd0::p1::Body::new(0, body));
        self
    }

    pub fn kind(mut self, kind: &str) -> Self {
        self.kind = Some(kind.to_string());
        self
    }

    /// returns a copy of Section after processing comments
    ///
    /// ## NOTE: This function is only called by [`ParsedDocument::ignore_comments()`]
    ///
    /// [`ParsedDocument::ignore_comments()`]: ftd0::p2::interpreter::ParsedDocument::ignore_comments
    pub fn remove_comments(&self) -> Option<Section> {
        if self.is_commented {
            return None;
        }
        Some(Section {
            name: self.name.to_string(),
            kind: self.kind.to_owned(),
            caption: self.caption.as_ref().and_then(|v| v.remove_comments()),
            headers: self.headers.clone().remove_comments(),
            body: self.body.as_ref().and_then(|v| v.remove_comments()),
            sub_sections: self
                .sub_sections
                .iter()
                .filter_map(|s| s.remove_comments())
                .collect::<Vec<ftd0::p1::Section>>(),
            is_commented: false,
            line_number: self.line_number,
            block_body: self.block_body,
        })
    }
}

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Body {
    pub line_number: usize,
    pub value: String,
}

impl Body {
    pub(crate) fn new(line_number: usize, value: &str) -> Body {
        Body {
            line_number,
            value: value.trim().to_string(),
        }
    }
    pub fn without_line_number(&self) -> Self {
        Body {
            line_number: 0,
            value: self.value.to_string(),
        }
    }

    pub(crate) fn remove_comments(&self) -> Option<Self> {
        let mut value = Some(self.value.to_owned());
        ftd0::p1::utils::remove_value_comment(&mut value);
        value.map(|value| Body {
            line_number: self.line_number,
            value,
        })
    }

    pub fn get_value(&self) -> String {
        self.value.to_string()
    }
}
