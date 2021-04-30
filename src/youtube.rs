use crate::document::ParseError;

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct YouTube {
    pub caption: Option<crate::Rendered>,
    pub src: String,
}

impl YouTube {
    pub fn to_p1(&self) -> crate::p1::Section {
        let mut p1 = crate::p1::Section::with_name("youtube").add_header("src", self.src.as_str());
        if let Some(ref c) = self.caption {
            p1 = p1.and_caption(c.original.as_str())
        }
        p1
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, ParseError> {
        if p1.body.is_some() {
            return Err(ParseError::ValidationError(
                "youtube can't have body".to_string(),
            ));
        }

        let mut yt = YouTube {
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

    pub fn with_src(mut self, src: &str) -> Self {
        self.src = src.to_string();
        self
    }
}

impl Default for YouTube {
    fn default() -> YouTube {
        YouTube {
            caption: None,
            src: "".to_string(),
        }
    }
}

impl ToString for YouTube {
    fn to_string(&self) -> String {
        format!(
            "-- youtube:{}\nsrc: {}\n",
            self.caption
                .as_ref()
                .map(|c| format!(" {}", c.original))
                .unwrap_or_else(|| "".to_string()),
            self.src.as_str(),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::document::f;
    use pretty_assertions::assert_eq;

    #[test]
    fn youtube() {
        assert_eq!(
            "-- youtube: foo\nsrc: foo.gif\n",
            crate::YouTube::default()
                .with_src("foo.gif")
                .with_caption("foo")
                .to_string()
        );

        f("-- youtube: \n", "P1Error: key not found: src");
        f("-- youtube:\nsrc: \n", "ValidationError: src is empty");
    }
}
