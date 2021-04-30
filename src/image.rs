use crate::document::{Align, ParseError};

#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Image {
    pub caption: Option<crate::Rendered>,
    pub src: String,
    // for mobile this will be shown when user taps on the image
    pub alt: Option<crate::Rendered>,
    pub align: Align,
    pub link: Option<String>,
    pub width: Option<i32>,
    pub height: Option<i32>,
    pub float: Option<bool>, // true == float left else float right
}

impl Default for Image {
    fn default() -> Image {
        Image {
            caption: None,
            src: "".to_string(),
            alt: None,
            align: crate::Align::Center,
            link: None,
            width: None,
            height: None,
            float: None,
        }
    }
}

impl ToString for Image {
    fn to_string(&self) -> String {
        format!(
            "-- image:{}\nsrc: {}\n{}{}{}{}{}{}",
            self.caption
                .as_ref()
                .map(|c| format!(" {}", c.original))
                .unwrap_or_else(|| "".to_string()),
            self.src.as_str(),
            self.alt
                .as_ref()
                .map(|v| format!("alt: {}\n", v.original))
                .unwrap_or_else(|| "".to_string()),
            self.align_str(),
            self.link
                .as_ref()
                .map(|v| format!("link: {}\n", v))
                .unwrap_or_else(|| "".to_string()),
            self.width
                .map(|v| format!("width: {}\n", v))
                .unwrap_or_else(|| "".to_string()),
            self.height
                .map(|v| format!("height: {}\n", v))
                .unwrap_or_else(|| "".to_string()),
            self.float_str()
        )
    }
}

impl Image {
    pub fn to_p1(&self) -> crate::p1::Section {
        let mut p1 = crate::p1::Section::with_name("image").add_header("src", self.src.as_str());
        if let Some(ref c) = self.caption {
            p1 = p1.and_caption(c.original.as_str())
        }
        if let Some(h) = self.height {
            p1 = p1.add_header("height", h.to_string().as_str());
        }
        if let Some(w) = self.width {
            p1 = p1.add_header("width", w.to_string().as_str());
        }
        if let Some(ref a) = self.alt {
            p1 = p1.add_header("alt", a.original.as_str());
        }
        if self.align != crate::Align::default() {
            p1 = p1.add_header("align", self.align.as_str());
        }

        p1
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, ParseError> {
        if p1.body.is_some() {
            return Err(ParseError::ValidationError(
                "image can't have body".to_string(),
            ));
        }

        let mut img = Image {
            src: p1.header.str("src")?.to_string(),
            height: p1.header.i32_optional("height")?,
            width: p1.header.i32_optional("width")?,
            float: p1.header.str_optional("float")?.map(|v| v == "left"),
            ..Default::default()
        };
        img.align = p1
            .header
            .str_with_default("align", img.align.as_str())?
            .parse()?;
        if let Some(ref caption) = p1.caption {
            img.caption = Some(crate::Rendered::line(caption.as_str()));
        }
        if let Some(alt) = p1.header.str_optional("alt")? {
            img.alt = Some(crate::Rendered::line(alt));
        }
        if let Some(link) = p1.header.str_optional("link")? {
            img.link = Some(link.to_string());
        }

        if img.src.trim().is_empty() {
            return Err(ParseError::ValidationError("src is empty".to_string()));
        }

        Ok(img)
    }

    fn float_str(&self) -> &'static str {
        match self.float {
            Some(true) => "float: left\n",
            Some(false) => "float: right\n",
            None => "",
        }
    }
    fn align_str(&self) -> &'static str {
        match self.align {
            Align::Left => "align: left\n",
            Align::Center => "",
            Align::Right => "align: right\n",
        }
    }

    pub fn with_caption(mut self, caption: &str) -> Self {
        self.caption = Some(crate::Rendered::line(caption));
        self
    }

    pub fn with_src(mut self, src: &str) -> Self {
        self.src = src.to_string();
        self
    }

    pub fn with_alt(mut self, alt: &str) -> Self {
        self.alt = Some(crate::Rendered::line(alt));
        self
    }

    pub fn with_align(mut self, align: crate::Align) -> Self {
        self.align = align;
        self
    }

    pub fn with_link(mut self, link: &str) -> Self {
        self.link = Some(link.to_string());
        self
    }

    pub fn with_width(mut self, width: i32) -> Self {
        self.width = Some(width);
        self
    }

    pub fn with_height(mut self, height: i32) -> Self {
        self.height = Some(height);
        self
    }

    pub fn with_float(mut self, float: bool) -> Self {
        self.float = Some(float);
        self
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn image() {
        assert_eq!(
            "-- image: foo\nsrc: foo.gif\n",
            crate::Image::default()
                .with_src("foo.gif")
                .with_caption("foo")
                .to_string()
        );

        assert_eq!(
            "-- image:\nsrc: foo.gif\nalt: alt text\n",
            crate::Image::default()
                .with_src("foo.gif")
                .with_alt("alt text")
                .to_string()
        );

        assert_eq!(
            "-- image:\nsrc: foo.gif\nalign: left\n",
            crate::Image::default()
                .with_src("foo.gif")
                .with_align(crate::Align::Left)
                .to_string()
        );

        assert_eq!(
            "-- image:\nsrc: foo.gif\nlink: https://www.google.com\n",
            crate::Image::default()
                .with_src("foo.gif")
                .with_link("https://www.google.com")
                .to_string()
        );

        assert_eq!(
            "-- image:\nsrc: foo.gif\nwidth: 80\n",
            crate::Image::default()
                .with_src("foo.gif")
                .with_width(80)
                .to_string()
        );

        assert_eq!(
            "-- image:\nsrc: foo.gif\nheight: 24\n",
            crate::Image::default()
                .with_src("foo.gif")
                .with_height(24)
                .to_string()
        );

        assert_eq!(
            "-- image:\nsrc: foo.gif\nfloat: left\n",
            crate::Image::default()
                .with_src("foo.gif")
                .with_float(true)
                .to_string()
        );

        p(
            &indoc!(
                "
                 -- image: some caption
                 src: img src
                 alt: alt tag
            "
            ),
            &vec![crate::Section::Image(
                crate::Image::default()
                    .with_caption("some caption")
                    .with_src("img src")
                    .with_align(super::Align::Center)
                    .with_alt("alt tag"),
            )],
        );

        p(
            &indoc!(
                "
                 -- image: some caption
                 src: img src
                 align: center
                 float: left
                 width: 20
                 height: 30
                 link: foo.com
            "
            ),
            &vec![crate::Section::Image(
                crate::Image::default()
                    .with_caption("some caption")
                    .with_src("img src")
                    .with_align(super::Align::Center)
                    .with_float(true)
                    .with_width(20)
                    .with_height(30)
                    .with_link("foo.com"),
            )],
        );

        f("-- image: \n", "P1Error: key not found: src");
        f("-- image:\nsrc: \n", "ValidationError: src is empty");
    }
}
