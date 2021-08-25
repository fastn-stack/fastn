pub use crate::p1::{Error, Header, Result};

#[derive(Debug, PartialEq, Default)]
pub struct SubSections(pub Vec<SubSection>);

#[derive(Debug, PartialEq, Default)]
pub struct SubSection {
    pub name: String,
    pub caption: Option<String>,
    pub header: Header,
    pub body: Option<String>,
}

impl SubSection {
    pub fn body(&self) -> Result<String> {
        match self.body {
            Some(ref body) => Ok(body.to_string()),
            None => Err(Error::NotFound {
                key: "body".to_string(),
            }),
        }
    }

    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            caption: None,
            header: Header::default(),
            body: None,
        }
    }

    pub fn and_caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn add_header(mut self, key: &str, value: &str) -> Self {
        self.header.0.push((key.to_string(), value.to_string()));
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
        self.body = Some(body.to_string());
        self
    }

    pub fn and_optional_body(mut self, body: &Option<String>) -> Self {
        self.body = body.as_ref().map(|v| v.to_string());
        self
    }
}

impl SubSections {
    pub fn by_name(&self, name: &str) -> Result<&SubSection> {
        for s in self.0.iter() {
            if s.name == name {
                return Ok(s);
            }
        }
        Err(Error::NotFound {
            key: name.to_string(),
        })
    }

    pub fn body_for(&self, name: &str) -> Result<String> {
        match self.by_name(name)?.body {
            Some(ref body) => Ok(body.to_string()),
            None => Err(Error::NotFound {
                key: name.to_string(),
            }),
        }
    }

    pub fn add_body(&mut self, name: &str, value: &str) {
        self.0.push(SubSection {
            name: name.to_string(),
            caption: None,
            header: Header::default(),
            body: Some(value.to_string()),
        })
    }

    pub fn add(&mut self, sub: SubSection) {
        self.0.push(sub)
    }
}
