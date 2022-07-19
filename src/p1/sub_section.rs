pub use ftd::p1::{Error, Header, Result};

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct SubSections(pub Vec<SubSection>);

impl SubSections {
    pub fn without_line_number(&self) -> Self {
        let mut subsections = vec![];
        for subsection in self.0.iter() {
            subsections.push(subsection.without_line_number());
        }
        SubSections(subsections)
    }
}

#[derive(Debug, PartialEq, Default, Clone, serde::Serialize, serde::Deserialize)]
pub struct SubSection {
    pub name: String,
    pub caption: Option<String>,
    pub header: Header,
    pub body: Option<(usize, String)>,
    pub is_commented: bool,
    pub line_number: usize,
}

impl SubSection {
    pub fn without_line_number(&self) -> Self {
        Self {
            name: self.name.to_string(),
            caption: self.caption.to_owned(),
            header: self.header.without_line_number(),
            body: self.body.to_owned().map(|v| (0, v.1)),
            is_commented: self.is_commented.to_owned(),
            line_number: 0,
        }
    }

    pub fn body_without_comment(&self) -> Option<(usize, String)> {
        let body = match &self.body {
            None => return None,
            Some(b) => b,
        };
        match body {
            _ if body.1.starts_with(r"\/") =>
            {
                #[allow(clippy::single_char_pattern)]
                Some((body.0, body.1.strip_prefix(r"\").expect("").to_string()))
            }
            _ if body.1.starts_with('/') => None,
            _ => Some((body.0, body.1.to_string())),
        }
    }

    pub fn remove_comments(&self) -> SubSection {
        let mut headers = vec![];
        for (i, k, v) in self.header.0.iter() {
            if !k.starts_with('/') {
                headers.push((i.to_owned(), k.to_string(), v.to_string()));
            }
        }

        let body = match &self.body {
            None => None,
            Some(body) => match body {
                _ if body.1.starts_with(r"\/") =>
                {
                    #[allow(clippy::single_char_pattern)]
                    Some((body.0, body.1.strip_prefix(r"\").expect("").to_string()))
                }
                _ if body.1.starts_with('/') => None,
                _ => self.body.clone(),
            },
        };

        SubSection {
            name: self.name.to_string(),
            caption: self.caption.to_owned(),
            header: Header(headers),
            body,
            is_commented: false,
            line_number: self.line_number,
        }
    }

    pub fn caption(&self, doc_id: &str) -> Result<String> {
        match self.caption {
            Some(ref v) => Ok(v.to_string()),
            None => Err(Error::ParseError {
                message: format!("caption is missing in {}", self.name),
                doc_id: doc_id.to_string(),
                line_number: self.line_number,
            }),
        }
    }

    pub fn body(&self, doc_id: &str) -> Result<String> {
        match self.body {
            Some(ref body) => Ok(body.1.to_string()),
            None => Err(Error::ParseError {
                message: format!("body is missing in {}", self.name),
                doc_id: doc_id.to_string(),
                line_number: self.line_number,
            }),
        }
    }

    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            caption: None,
            header: Header::default(),
            body: None,
            is_commented: false,
            line_number: 0,
        }
    }

    pub fn and_caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn add_header(mut self, key: &str, value: &str) -> Self {
        self.header.0.push((0, key.to_string(), value.to_string()));
        self
    }

    pub fn add_optional_header_bool(mut self, key: &str, value: Option<bool>) -> Self {
        if let Some(v) = value {
            self = self.add_header(key, v.to_string().as_str());
        }
        self
    }

    pub fn add_optional_header(mut self, key: &str, value: &Option<String>) -> Self {
        if let Some(v) = value {
            self = self.add_header(key, v.as_str());
        }
        self
    }

    pub fn add_header_if_not_equal<T>(self, key: &str, value: T, reference: T) -> Self
    where
        T: ToString + std::cmp::PartialEq,
    {
        if value != reference {
            self.add_header(key, value.to_string().as_str())
        } else {
            self
        }
    }

    pub fn and_body(mut self, body: &str) -> Self {
        self.body = Some((0, body.to_string()));
        self
    }

    pub fn and_optional_body(mut self, body: &Option<String>) -> Self {
        self.body = body.as_ref().map(|v| (0, v.to_string()));
        self
    }
}

impl SubSections {
    pub fn by_name(&self, line_number: usize, name: &str, doc_id: &str) -> Result<&SubSection> {
        for s in self.0.iter() {
            if s.is_commented {
                continue;
            }

            if s.name == name {
                return Ok(s);
            }
        }
        Err(Error::NotFound {
            doc_id: doc_id.to_string(),
            line_number,
            key: name.to_string(),
        })
    }

    pub fn body_for(&self, line_number: usize, name: &str, doc_id: &str) -> Result<String> {
        match self.by_name(line_number, name, doc_id)?.body {
            Some(ref body) => Ok(body.1.to_string()),
            None => Err(Error::NotFound {
                doc_id: doc_id.to_string(),
                line_number,
                key: name.to_string(),
            }),
        }
    }

    pub fn add_body(&mut self, name: &str, value: &str) {
        self.0.push(SubSection {
            name: name.to_string(),
            caption: None,
            header: Header::default(),
            body: Some((0, value.to_string())),
            is_commented: false,
            line_number: 0,
        })
    }

    pub fn add(&mut self, sub: SubSection) {
        self.0.push(sub)
    }
}
