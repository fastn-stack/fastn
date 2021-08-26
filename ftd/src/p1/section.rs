pub use crate::p1::{Error, Header, Result, SubSection, SubSections};

#[derive(Debug, PartialEq, Clone, Default, serde::Serialize, serde::Deserialize)]
pub struct Section {
    pub name: String,
    pub caption: Option<String>,
    pub header: Header,
    pub body: Option<String>,
    pub sub_sections: SubSections,
}

impl Section {
    pub fn caption(&self) -> Result<String> {
        match self.caption {
            Some(ref v) => Ok(v.to_string()),
            None => Err(Error::InvalidInput {
                message: "caption is missing".to_string(),
                context: "".to_string(),
            }),
        }
    }

    pub fn body(&self) -> Result<String> {
        match self.body {
            Some(ref v) => Ok(v.to_string()),
            None => Err(Error::InvalidInput {
                message: "body is missing".to_string(),
                context: "".to_string(),
            }),
        }
    }

    pub fn assert_missing(&self, key: &str) -> Result<()> {
        if self.header.str_optional(key)?.is_some() {
            return Err(Error::InvalidInput {
                message: format!("'{}' is not expected", key),
                context: "".to_string(),
            });
        }

        Ok(())
    }

    pub fn with_name(name: &str) -> Self {
        Self {
            name: name.to_string(),
            caption: None,
            header: Header::default(),
            body: None,
            sub_sections: SubSections::default(),
        }
    }

    pub fn and_caption(mut self, caption: &str) -> Self {
        self.caption = Some(caption.to_string());
        self
    }

    pub fn and_optional_caption(mut self, value: &Option<ftd_rt::Rendered>) -> Self {
        if let Some(v) = value {
            self = self.and_caption(v.original.as_str());
        }
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
    pub fn add_optional_header_i32(mut self, key: &str, value: &Option<i32>) -> Self {
        if let Some(v) = value {
            self = self.add_header(key, v.to_string().as_str());
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

    pub fn add_optional_header(mut self, key: &str, value: &Option<String>) -> Self {
        if let Some(v) = value {
            self = self.add_header(key, v.as_str());
        }
        self
    }

    pub fn and_body(mut self, body: &str) -> Self {
        self.body = Some(body.to_string());
        self
    }

    pub fn and_optional_body(mut self, body: &Option<String>) -> Self {
        self.body = body.as_ref().map(|v| v.to_string());
        self
    }

    pub fn add_sub_section(mut self, sub: SubSection) -> Self {
        self.sub_sections.0.push(sub);
        self
    }

    pub fn sub_section_by_name(&self, name: &str) -> crate::p1::Result<&crate::p1::SubSection> {
        let mut count = 0;
        for s in self.sub_sections.0.iter() {
            if s.name == name {
                count += 1;
            }
        }
        if count > 1 {
            return Err(crate::p1::Error::MoreThanOneSubSections {
                key: name.to_string(),
            });
        }

        for s in self.sub_sections.0.iter() {
            if s.name == name {
                return Ok(s);
            }
        }

        Err(crate::p1::Error::NotFound {
            key: name.to_string(),
        })
    }

    #[cfg(test)]
    pub(crate) fn list(self) -> Vec<Self> {
        vec![self]
    }
}
