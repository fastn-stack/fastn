use crate::document::ParseError;

#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct YouTube {
    pub id: Option<String>,
    pub caption: Option<crate::Rendered>,
    pub src: String,
}

impl YouTube {
    pub fn to_p1(&self) -> crate::p1::Section {
        crate::p1::Section::with_name("youtube")
            .and_optional_caption(&self.caption)
            .add_optional_header("id", &self.id)
            .add_header("src", self.src.as_str())
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, ParseError> {
        if p1.body.is_some() {
            return Err(ParseError::ValidationError(
                "youtube can't have body".to_string(),
            ));
        }

        let mut yt = YouTube {
            id: p1.header.string_optional("id")?,
            src: p1.header.str("src")?.to_string(),
            ..Default::default()
        };

        if let Some(ref caption) = p1.caption {
            yt.caption = Some(crate::Rendered::line(caption.as_str()));
        }

        if yt.src.trim().is_empty() {
            return Err(ParseError::ValidationError("src is empty".to_string()));
        }

        Ok(yt)
    }

    pub fn with_caption(mut self, caption: &str) -> Self {
        self.caption = Some(crate::Rendered::line(caption));
        self
    }

    pub fn with_id(mut self, id: &str) -> Self {
        self.id = Some(id.to_string());
        self
    }

    pub fn with_src(mut self, src: &str) -> Self {
        self.src = src.to_string();
        self
    }
}

impl Default for YouTube {
    fn default() -> YouTube {
        YouTube {
            id: None,
            caption: None,
            src: "".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::document::f;
    use pretty_assertions::assert_eq;

    #[test]
    fn youtube() {
        assert_eq!(
            "-- youtube: foo\nid: temp\nsrc: foo.gif\n",
            crate::YouTube::default()
                .with_src("foo.gif")
                .with_caption("foo")
                .with_id("temp")
                .to_p1()
                .to_string()
        );

        f("-- youtube: \n", "P1Error: key not found: src");
        f("-- youtube:\nsrc: \n", "ValidationError: src is empty");
    }
}
