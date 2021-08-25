use crate::document::ParseError;

#[derive(PartialEq, Debug, Default, Clone, serde_derive::Serialize)]
pub struct Heading {
    pub level: u8,
    pub title: crate::Rendered,
    pub id: crate::ValueWithDefault<String>,
}

impl Heading {
    pub fn new(level: u8, title: &str) -> Self {
        Heading {
            level,
            title: crate::Rendered::line(title),
            id: crate::ValueWithDefault::Default(slug::slugify(title)),
        }
    }

    pub fn with_level(p1: &crate::p1::Section, level: u8) -> Result<Self, ParseError> {
        let title = match p1.caption {
            Some(ref c) => c.trim().to_string(),
            None => return Err(ParseError::ValidationError("heading is empty".to_string())),
        };

        Ok(Heading {
            level,
            title: crate::Rendered::line(title.as_str()),
            id: match p1.header.string_optional("id")? {
                Some(v) => crate::ValueWithDefault::Found(v),
                None => crate::ValueWithDefault::Default(slug::slugify(title)),
            },
        })
    }

    pub fn to_p1(&self) -> crate::p1::Section {
        let p1 = crate::p1::Section::with_name(format!("h{}", self.level).as_str())
            .and_caption(self.title.original.as_str());
        match self.id {
            crate::ValueWithDefault::Found(ref v) => p1.add_header("id", v.as_str()),
            crate::ValueWithDefault::Default(_) => p1,
        }
    }

    pub fn from_p1(p1: &crate::p1::Section) -> Result<Self, ParseError> {
        match p1.caption {
            Some(ref c) => {
                let mut parts = c.splitn(2, ' ');
                match (parts.next(), parts.next()) {
                    (Some(l), Some(r)) => {
                        let (title, level) = match l.parse() {
                            Ok(l) => (r.trim().to_string(), l),
                            Err(_) => (c.trim().to_string(), 0),
                        };

                        Ok(Heading {
                            level,
                            title: crate::Rendered::line(title.as_str()),
                            id: match p1.header.string_optional("id")? {
                                Some(v) => crate::ValueWithDefault::Found(v),
                                None => crate::ValueWithDefault::Default(slug::slugify(title)),
                            },
                        })
                    }
                    _ => Err(ParseError::ValidationError(
                        "heading must have level and title".to_string(),
                    )),
                }
            }
            None => Err(ParseError::ValidationError("heading is empty".to_string())),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::prelude::*;

    #[test]
    fn headings() {
        assert_eq!(
            "-- h27: hello world\n",
            crate::Heading {
                level: 27,
                title: crate::Rendered::line("hello world"),
                id: crate::ValueWithDefault::Default("hello-world".to_string()),
            }
            .to_p1()
            .to_string()
        );

        p(
            &indoc::indoc!(
                "
                 -- heading: hello world2
            "
            ),
            &vec![crate::Section::Heading(crate::Heading {
                level: 0,
                title: crate::Rendered::line("hello world2"),
                id: crate::ValueWithDefault::Default("hello-world2".to_string()),
            })],
        );

        // NOT: this is no longer supported
        // p(
        //     &indoc::indoc!(
        //         "
        //          -- heading: hello world2
        //          this is the body
        //
        //          multi line
        //     "
        //     ),
        //     Module {
        //         sections: vec![
        //             Section::Heading(Heading {
        //                 level: 0,
        //                 title: crate::Rendered::line("hello world2"),
        //             }),
        //             Section::Markdown(Markdown {
        //                 body: crate::Rendered::from("this is the body\n\nmulti line"),
        //                 hard_breaks: false,
        //                 auto_links: true,
        //                 align: Align::default(),
        //                 direction: TextDirection::default(),
        //                 two_columns: false,
        //             }),
        //         ],
        //     },
        // );

        // NOT: this is no longer supported
        // p(
        //     &indoc::indoc!(
        //         "
        //          -- heading: hello world2
        //          this is the body
        //
        //          multi line
        //          -- heading: hello 2 world
        //     "
        //     ),
        //     Module {
        //         sections: vec![
        //             Section::Heading(Heading {
        //                 level: 0,
        //                 title: crate::Rendered::line("hello world2"),
        //             }),
        //             Section::Markdown(Markdown {
        //                 body: crate::Rendered::from("this is the body\n\nmulti line"),
        //                 hard_breaks: false,
        //                 auto_links: true,
        //                 align: Align::default(),
        //                 direction: TextDirection::default(),
        //                 two_columns: false,
        //             }),
        //             Section::Heading(Heading {
        //                 level: 0,
        //                 title: crate::Rendered::line("hello 2 world"),
        //             }),
        //         ],
        //     },
        // );

        p(
            // should this be syntax error?
            &indoc::indoc!(
                "
                 -- heading:12 hello world
            "
            ),
            &vec![crate::Section::Heading(crate::Heading {
                level: 12,
                title: crate::Rendered::line("hello world"),
                id: crate::ValueWithDefault::Default("hello-world".to_string()),
            })],
        );

        // NOTE: this is no longer supported
        // p(
        //     // level must follow right after colon, without space, else its part of title
        //     &indoc::indoc!(
        //         "
        //          -- heading: 12 hello world
        //     "
        //     ),
        //     Module {
        //         sections: vec![Section::Heading(Heading {
        //             level: 0,
        //             title: crate::Rendered::line("12 hello world"),
        //         })],
        //     },
        // );
        f("-- heading: ", "ValidationError: heading is empty");
        p(
            "-- heading:1 yo\n\nhello",
            &vec![
                crate::Section::Heading(crate::Heading {
                    level: 1,
                    title: crate::Rendered::line("yo"),
                    id: crate::ValueWithDefault::Default("yo".to_string()),
                }),
                crate::Section::Markdown(crate::Markdown {
                    id: None,
                    body: crate::Rendered::from("hello"),
                    caption: None,
                    hard_breaks: false,
                    auto_links: true,
                    align: crate::Align::default(),
                    direction: crate::TextDirection::default(),
                    two_columns: false,
                    collapsed: false,
                }),
            ],
        );
    }
}
