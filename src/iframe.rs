use crate::document::ParseError;

#[derive(PartialEq, Debug, Clone, Serialize, Default)]
pub struct IFrame {
    pub caption: Option<crate::Rendered>,
    pub src: String,
    pub height: Option<i32>,
    pub width: Option<i32>,
    pub desktop: Option<String>,
    pub tablet: Option<String>,
    pub mobile: Option<String>,
}

impl IFrame {
    pub fn to_p1(&self) -> crate::p1::Section {
        let mut p1 = crate::p1::Section::with_name("iframe")
            .add_header("src", self.src.as_str())
            .add_optional_header("desktop", &self.desktop)
            .add_optional_header("tablet", &self.tablet)
            .add_optional_header("mobile", &self.mobile);
        if let Some(ref c) = self.caption {
            p1 = p1.and_caption(c.original.as_str())
        }
        if let Some(h) = self.height {
            p1 = p1.add_header("height", h.to_string().as_str());
        }
        if let Some(w) = self.width {
            p1 = p1.add_header("width", w.to_string().as_str());
        }
        p1
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, ParseError> {
        if p1.body.is_some() {
            return Err(ParseError::ValidationError(
                "iframe can't have body".to_string(),
            ));
        }

        let f = IFrame {
            src: p1.header.str("src")?.to_string(),
            height: p1.header.i32_optional("height")?,
            width: p1.header.i32_optional("width")?,
            caption: p1
                .caption
                .as_ref()
                .map(|c| crate::Rendered::line(c.as_str())),
            desktop: p1.header.string_optional("desktop")?,
            tablet: p1.header.string_optional("tablet")?,
            mobile: p1.header.string_optional("mobile")?,
        };

        if f.src.trim().is_empty() {
            return Err(ParseError::ValidationError("src is empty".to_string()));
        }

        Ok(f)
    }

    pub fn with_caption(mut self, caption: &str) -> Self {
        self.caption = Some(crate::Rendered::line(caption));
        self
    }

    pub fn with_src(mut self, src: &str) -> Self {
        self.src = src.to_string();
        self
    }

    pub fn with_height(mut self, height: i32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_width(mut self, width: i32) -> Self {
        self.width = Some(width);
        self
    }
}

impl ToString for IFrame {
    fn to_string(&self) -> String {
        format!(
            "-- iframe:{}\nsrc: {}\n{}{}",
            self.caption
                .as_ref()
                .map(|c| format!(" {}", c.original))
                .unwrap_or_else(|| "".to_string()),
            self.src.as_str(),
            self.height
                .map(|v| format!("height: {}\n", v))
                .unwrap_or_else(|| "".to_string()),
            self.width
                .map(|v| format!("width: {}\n", v))
                .unwrap_or_else(|| "".to_string()),
        )
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;

    #[test]
    fn iframe() {
        assert_eq!(
            "-- iframe: google\nsrc: https://google.com\n",
            crate::IFrame::default()
                .with_src("https://google.com")
                .with_caption("google")
                .to_string()
        );

        assert_eq!(
            "-- iframe: google\nsrc: https://google.com\nheight: 24\n",
            crate::IFrame::default()
                .with_src("https://google.com")
                .with_caption("google")
                .with_height(24)
                .to_string()
        );

        assert_eq!(
            "-- iframe: google\nsrc: https://google.com\nwidth: 24\n",
            crate::IFrame::default()
                .with_src("https://google.com")
                .with_caption("google")
                .with_width(24)
                .to_string()
        );

        f("-- iframe: \n", "P1Error: key not found: src");
        f("-- iframe:\nsrc: \n", "ValidationError: src is empty");
    }
}
