#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct Rst {
    pub id: Option<String>,
    pub body: crate::Rendered,
    pub caption: Option<crate::Rendered>,
    pub collapsed: bool,
}

impl Default for Rst {
    fn default() -> Self {
        Rst {
            id: None,
            body: crate::Rendered::default(),
            caption: None,
            collapsed: false,
        }
    }
}

impl Rst {
    pub fn to_p1(&self) -> crate::p1::Section {
        let mut p1 = crate::p1::Section::with_name("rst")
            .and_optional_caption(&self.caption)
            .and_body(self.body.original.as_str());

        if let Some(id) = &self.id {
            p1 = p1.add_header("id", id)
        }

        if self.collapsed {
            p1 = p1.add_header("collapsed", "true");
        }
        p1
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        let mut m = Rst::default();
        if let Some(ref c) = p1.caption {
            m.caption = Some(crate::Rendered::line(c));
        }
        match p1.body {
            Some(ref b) => m.body = crate::Rendered::rst(b),
            _ => {
                return Err(crate::document::ParseError::ValidationError(
                    "body must be present for rst".to_string(),
                ))
            }
        }
        m.id = p1.header.string_optional("id")?;
        m.collapsed = p1.header.bool_with_default("collapsed", m.collapsed)?;
        Ok(m)
    }

    pub fn from_body(body: &str) -> Self {
        Self {
            body: crate::Rendered::rst(body),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn rst() {
        assert_eq!(
            "-- rst:\n\nhello world is\n\n    not enough\n\n    lol\n",
            crate::Rst {
                id: None,
                body: crate::Rendered::rst("hello world is\n\n    not enough\n\n    lol"),
                collapsed: false,
                caption: None,
            }
            .to_p1()
            .to_string()
        );

        p(
            &indoc::indoc!(
                "
                 -- rst:
                 id: temp
                 hello world is

                     not enough

                     lol
            "
            ),
            &vec![crate::Section::Rst(crate::Rst {
                id: Some("temp".to_string()),
                body: crate::Rendered::rst("hello world is\n\n    not enough\n\n    lol"),
                collapsed: false,
                caption: None,
            })],
        );

        f("-- rst:", "ValidationError: body must be present for rst");

        f(
            "-- rst:\n-- rst:",
            "ValidationError: body must be present for rst",
        );

        f(
            "-- rst:  \n-- rst:",
            "ValidationError: body must be present for rst",
        );
    }
}
