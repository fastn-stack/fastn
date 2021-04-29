#[derive(PartialEq, Debug, Clone, Serialize)]
pub struct Markdown {
    pub body: crate::Rendered,
    pub caption: Option<crate::Rendered>,
    pub hard_breaks: bool,
    pub auto_links: bool,
    pub align: crate::Align,
    pub direction: crate::TextDirection,
    pub two_columns: bool,
    pub collapsed: bool,
}

impl ToString for Markdown {
    fn to_string(&self) -> String {
        format!(
            "-- markdown:{}\n{}{}{}{}{}{}\n{}",
            self.caption_string(),
            self.auto_links_str(),
            self.hard_breaks_str(),
            self.align_str(),
            self.direction_str(),
            self.two_columns_str(),
            self.collapsed_str(),
            self.body.original,
        )
    }
}

impl Default for Markdown {
    fn default() -> Markdown {
        Markdown {
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

        if self.hard_breaks {
            p1 = p1.add_header("hard_breaks", "true");
        }
        if self.two_columns {
            p1 = p1.add_header("two_columns", "true");
        }
        if self.collapsed {
            p1 = p1.add_header("collapsed", "true");
        }
        if self.auto_links {
            p1 = p1.add_header("auto_links", "true");
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

    fn two_columns_str(&self) -> &'static str {
        if self.two_columns {
            "two_columns: true\n"
        } else {
            ""
        }
    }

    fn caption_string(&self) -> String {
        match self.caption {
            Some(ref c) => format!(" {}", c.original.as_str()),
            None => "".to_string(),
        }
    }

    fn collapsed_str(&self) -> &'static str {
        if self.collapsed {
            "collapsed: true\n"
        } else {
            ""
        }
    }

    fn hard_breaks_str(&self) -> &'static str {
        if self.hard_breaks {
            "hard_breaks: true\n"
        } else {
            ""
        }
    }
    fn auto_links_str(&self) -> &'static str {
        if self.auto_links {
            ""
        } else {
            "auto_links: false\n"
        }
    }
    fn align_str(&self) -> &'static str {
        match self.align {
            crate::Align::Left => "",
            crate::Align::Center => "align: center\n",
            crate::Align::Right => "align: right\n",
        }
    }
    fn direction_str(&self) -> &'static str {
        match self.direction {
            crate::TextDirection::LeftToRight => "",
            crate::TextDirection::RightToLeft => "direction: rtl\n",
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
            "-- markdown:\n\nhello world is\n\n    not enough\n\n    lol",
            crate::Markdown {
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_string()
        );

        assert_eq!(
            "-- markdown:\nhard_breaks: true\n\nhello world is\n\n    not enough\n\n    lol",
            crate::Markdown {
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: true,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_string()
        );

        assert_eq!(
            "-- markdown:\nauto_links: false\n\nhello world is\n\n    not enough\n\n    lol",
            crate::Markdown {
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: false,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_string()
        );

        assert_eq!(
            "-- markdown:\nalign: right\n\nhello world is\n\n    not enough\n\n    lol",
            crate::Markdown {
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::Right,
                direction: crate::TextDirection::default(),
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_string()
        );

        assert_eq!(
            "-- markdown:\ndirection: rtl\n\nhello world is\n\n    not enough\n\n    lol",
            crate::Markdown {
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::RightToLeft,
                two_columns: false,
                collapsed: false,
                caption: None,
            }
            .to_string()
        );

        assert_eq!(
            "-- markdown:\ntwo_columns: true\n\nhello world is\n\n    not enough\n\n    lol",
            crate::Markdown {
                body: crate::Rendered::from("hello world is\n\n    not enough\n\n    lol"),
                hard_breaks: false,
                auto_links: true,
                align: crate::Align::default(),
                direction: crate::TextDirection::default(),
                two_columns: true,
                collapsed: false,
                caption: None,
            }
            .to_string()
        );

        p(
            &indoc!(
                "
                 -- markdown:
                 hello world is

                     not enough

                     lol
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
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
            &indoc!(
                "
                 -- markdown:
                 auto_links: false
                 hard_breaks: true

                 hello world is

                     not enough

                     lol
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
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
            &indoc!(
                "
                 -- markdown:
                 direction: rtl
                 align: center

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
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
            &indoc!(
                "
                 -- markdown:
                 direction: rtl
                 align: centre

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
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
            &indoc!(
                "
                 -- markdown:
                 direction: ltr
                 align: right

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
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
            &indoc!(
                "
                 -- markdown:
                 two_columns: true

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
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
            &indoc!(
                "
                 -- markdown:
                 collapsed: true

                 hello world
            "
            ),
            &vec![crate::Section::Markdown(crate::Markdown {
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
