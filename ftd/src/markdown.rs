#[derive(PartialEq, Debug, Clone, serde_derive::Serialize)]
pub struct Markdown {
    pub id: Option<String>,
    pub body: crate::Rendered,
    pub caption: Option<crate::Rendered>,
    pub hard_breaks: bool,
    pub auto_links: bool,
    pub align: crate::Align,
    pub direction: crate::TextDirection,
    pub two_columns: bool,
    pub collapsed: bool,
}

impl Default for Markdown {
    fn default() -> Markdown {
        Markdown {
            id: None,
            body: crate::Rendered::default(),
            caption: None,
            hard_breaks: false,
            auto_links: true,
            align: crate::Align::default(),
            direction: crate::TextDirection::default(),
            two_columns: false,
            collapsed: false,
        }
    }
}

impl Markdown {
    pub fn to_p1(&self) -> crate::p1::Section {
        let mut p1 = crate::p1::Section::with_name("markdown")
            .and_optional_caption(&self.caption)
            .and_body(self.body.original.as_str());

        if let Some(id) = &self.id {
            p1 = p1.add_header("id", id)
        }

        if self.hard_breaks {
            p1 = p1.add_header("hard_breaks", "true");
        }
        if self.two_columns {
            p1 = p1.add_header("two_columns", "true");
        }
        if self.collapsed {
            p1 = p1.add_header("collapsed", "true");
        }
        if !self.auto_links {
            p1 = p1.add_header("auto_links", "false");
        }
        if self.direction != crate::TextDirection::default() {
            p1 = p1.add_header("direction", self.direction.as_str());
        }
        if self.align != crate::Align::default() {
            p1 = p1.add_header("align", self.align.as_str());
        }
        p1
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, crate::document::ParseError> {
        let mut m = Markdown::default();
        if let Some(ref c) = p1.caption {
            m.caption = Some(crate::Rendered::line(c));
        }
        match p1.body {
            Some(ref b) => m.body = crate::Rendered::from(b),
            None => {
                return Err(crate::document::ParseError::ValidationError(
                    "body must be present for markdown".to_string(),
                ))
            }
        }
        m.id = p1.header.string_optional("id")?;
        m.hard_breaks = p1.header.bool_with_default("hard_breaks", m.hard_breaks)?;
        m.auto_links = p1.header.bool_with_default("auto_links", m.auto_links)?;
        m.align = p1
            .header
            .str_with_default("align", m.align.as_str())?
            .parse()?;
        m.direction = p1
            .header
            .str_with_default("direction", m.direction.as_str())?
            .parse()?;
        m.two_columns = p1.header.bool_with_default("two_columns", m.two_columns)?;
        m.collapsed = p1.header.bool_with_default("collapsed", m.collapsed)?;
        Ok(m)
    }

    pub fn from_body(body: &str) -> Self {
        Self {
            body: crate::Rendered::from(body),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use pretty_assertions::assert_eq;

    #[test]
    fn markdown() {
        assert_eq!(
            "-- markdown:\n\nhello world is\n\n    not enough\n\n    lol\n",
            crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_p1()
            .to_string()
        );

        assert_eq!(
            "-- markdown:\nhard_breaks: true\n\nhello world is\n\n    not enough\n\n    lol\n",
            crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: true,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_p1()
            .to_string()
        );

        assert_eq!(
            "-- markdown:\nauto_links: false\n\nhello world is\n\n    not enough\n\n    lol\n",
            crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: false,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_p1()
            .to_string()
        );

        assert_eq!(
            "-- markdown:\nalign: right\n\nhello world is\n\n    not enough\n\n    lol\n",
            crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::Right,
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_p1()
            .to_string()
        );

        assert_eq!(
            "-- markdown:\ndirection: rtl\n\nhello world is\n\n    not enough\n\n    lol\n",
            crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::RightToLeft,
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_p1()
            .to_string()
        );

        assert_eq!(
            "-- markdown:\ntwo_columns: true\n\nhello world is\n\n    not enough\n\n    lol\n",
            crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: true,
                collapsed: false,
                caption: None,
            }
            .to_p1()
            .to_string()
        );

        p(
            &indoc::indoc!(
                "
                 -- markdown:
                 id: temp
                 hello world is

                     not enough

                     lol
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
                id: Some("temp".to_string()),
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            })],
        );

        p(
            &indoc::indoc!(
                "
                 -- markdown:
                 id: temp
                 auto_links: false
                 hard_breaks: true

                 hello world is

                     not enough

                     lol
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
                id: Some("temp".to_string()),
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: true,
                auto_links: false,
                align: crate::Align::Left,
                direction: crate::TextDirection::LeftToRight,
                two_columns: false,
                collapsed: false,
                caption: None,
            })],
        );

        p(
            &indoc::indoc!(
                "
                 -- markdown:
                 direction: rtl
                 align: center

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::Center,
                direction: crate::TextDirection::RightToLeft,
                two_columns: false,
                collapsed: false,
                caption: None,
            })],
        );

        p(
            &indoc::indoc!(
                "
                 -- markdown:
                 direction: rtl
                 align: centre

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::Center,
                direction: crate::TextDirection::RightToLeft,
                two_columns: false,
                collapsed: false,
                caption: None,
            })],
        );

        p(
            &indoc::indoc!(
                "
                 -- markdown:
                 direction: ltr
                 align: right

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::Right,
                direction: crate::TextDirection::LeftToRight,
                two_columns: false,
                collapsed: false,
                caption: None,
            })],
        );

        p(
            &indoc::indoc!(
                "
                 -- markdown:
                 two_columns: true

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::Left,
                direction: crate::TextDirection::LeftToRight,
                two_columns: true,
                collapsed: false,
                caption: None,
            })],
        );

        p(
            &indoc::indoc!(
                "
                 -- markdown:
                 collapsed: true

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
                id: None,
                body: crate::Rendered::from("hello world"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::Left,
                direction: crate::TextDirection::LeftToRight,
                two_columns: false,
                collapsed: true,
                caption: None,
            })],
        );

        f(
            "-- markdown:",
            "ValidationError: body must be present for markdown",
        );

        f(
            "-- markdown:\n-- markdown:",
            "ValidationError: body must be present for markdown",
        );

        f(
            "-- markdown:  \n-- markdown:",
            "ValidationError: body must be present for markdown",
        );
    }
}
