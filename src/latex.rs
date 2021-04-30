#[derive(PartialEq, Debug, Clone, Serialize, Default)]
pub struct Latex {
    pub caption: Option<crate::Rendered>,
    pub body: crate::Rendered,
}

impl ToString for Latex {
    fn to_string(&self) -> String {
        format!(
            "-- latex:{}\n\n{}",
            self.caption
                .as_ref()
                .map_or_else(|| "", |r| r.original.as_str()),
            self.body.original
        )
    }
}

impl Latex {
    pub fn to_p1(&self) -> crate::p1::Section {
        crate::p1::Section::with_name("latex")
            .and_body(self.body.original.as_str())
            .and_optional_caption(&self.caption)
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        let body = match p1.body {
            Some(ref b) => crate::Rendered::latex(b)?,
            None => {
                return Err(crate::document::ParseError::ValidationError(
                    "body must be present for latex".to_string(),
                ))
            }
        };

        Ok(Latex {
            caption: p1
                .caption
                .as_ref()
                .map(|s| crate::Rendered::line(s.as_str())),
            body,
        })
    }

    pub fn with_caption(mut self, caption: &str) -> Self {
        self.caption = Some(crate::Rendered::line(caption));
        self
    }

    pub fn with_body(mut self, body: &str) -> Result<Self, crate::document::ParseError> {
        self.body = crate::Rendered::latex(body)?;
        Ok(self)
    }
}

#[cfg(test)]
mod tests {
    use crate::prelude::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn latex() {
        assert_eq!(
            "-- latex:\n\n\\int_0^\\infty x^2 dx\n",
            crate::Latex::default()
                .with_body("\\int_0^\\infty x^2 dx\n")
                .unwrap()
                .to_string()
        );
        p(
            &indoc!(
                "
                -- latex: some caption

                \\begin{Bmatrix}
                a & b \\
                c & d
                \\end{Bmatrix}
            "
            ),
            &vec![crate::Section::Latex(
                crate::Latex::default()
                    .with_caption("some caption")
                    .with_body("\\begin{Bmatrix}\na & b \\\nc & d\n\\end{Bmatrix}")
                    .unwrap(),
            )],
        );
    }
}
