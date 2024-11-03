pub fn to_string(p1: &[ftd::ftd2021::p1::Section]) -> String {
    p1.iter()
        .map(|v| v.to_string().trim().to_string())
        .collect::<Vec<String>>()
        .join("\n\n\n")
}

impl std::fmt::Display for ftd::ftd2021::p1::Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_commented {
            write!(f, "/-- {}:", self.name.as_str())?;
        } else {
            write!(f, "-- {}:", self.name.as_str())?;
        }
        if let Some(ref caption) = self.caption {
            write!(f, " {}", caption)?;
        }

        for (_, k, v) in self.header.0.iter() {
            write!(f, "\n{}: {}", k, v)?;
        }

        writeln!(f)?;

        if let Some(ref body) = self.body {
            write!(f, "\n{}\n", escape_body(&body.1))?;
        }

        for sub in self.sub_sections.0.iter() {
            write!(f, "\n{}", sub)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for ftd::ftd2021::p1::SubSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.is_commented {
            write!(f, "/--- {}:", self.name.as_str())?;
        } else {
            write!(f, "--- {}:", self.name.as_str())?;
        }
        if let Some(ref caption) = self.caption {
            write!(f, " {}", caption)?;
        }

        for (_, k, v) in self.header.0.iter() {
            write!(f, "\n{}: {}", k, v)?;
        }

        if let Some(ref body) = self.body {
            write!(f, "\n\n{}", escape_body(&body.1))?;
        }

        writeln!(f)
    }
}

fn escape_body(body: &str) -> String {
    fn remove_newline_start(body: String) -> String {
        match body.strip_prefix('\n') {
            Some(body) => remove_newline_start(body.to_string()),
            None => body,
        }
    }

    let body = "\n".to_string() + body;
    let body = body
        .replace("\n-- ", "\n\\-- ")
        .replace("\n--- ", "\n\\--- ");

    remove_newline_start(body).trim_end().to_string()
}

#[cfg(test)]
mod test {
    use {indoc::indoc, pretty_assertions::assert_eq};

    // macro

    #[test]
    pub fn test_comments() {
        assert_eq!(
            indoc!(
                "/-- ftd.row:
                /color: red

                --- ftd.text:

                hello world"
            ),
            super::to_string(
                &ftd::ftd2021::p1::parse(
                    indoc!(
                        "
                    /-- ftd.row:
                    /color: red

                    --- ftd.text:



                    hello world
                    "
                    ),
                    "foo"
                )
                .expect("Cannot unresolved to section")
            )
        );
    }

    #[test]
    pub fn subsection_formatter() {
        assert_eq!(
            indoc!(
                "-- ftd.row:

                --- ftd.text:

                hello world"
            ),
            super::to_string(
                &ftd::ftd2021::p1::parse(
                    indoc!(
                        "
                -- ftd.row:

                --- ftd.text:



                hello world
                "
                    ),
                    "foo"
                )
                .expect("Cannot unresolved to section")
            )
        );

        assert_eq!(
            indoc!(
                "
             -- ftd.text:

                hello world
                hello world again"
            ),
            super::to_string(
                &ftd::ftd2021::p1::parse(
                    indoc!(
                        "
                     -- ftd.text:





                        hello world
                        hello world again
                     "
                    ),
                    "foo"
                )
                .expect("Cannot unresolved to section")
            )
        );
    }

    #[test]
    pub fn to_string() {
        assert_eq!(
            indoc!(
                "
            -- foo:
            key: value

            body ho

            --- dodo: foo
            foo: bar

            --- dodo:
            foo: bar


            -- bar:

            bar body"
            ),
            super::to_string(&vec![
                ftd::ftd2021::p1::Section::with_name("foo")
                    .and_body("body ho")
                    .add_header("key", "value")
                    .add_sub_section(
                        ftd::ftd2021::p1::SubSection::with_name("dodo")
                            .and_caption("foo")
                            .add_header("foo", "bar"),
                    )
                    .add_sub_section(
                        ftd::ftd2021::p1::SubSection::with_name("dodo").add_header("foo", "bar")
                    ),
                ftd::ftd2021::p1::Section::with_name("bar").and_body("bar body")
            ]),
        );

        assert_eq!(
            indoc!(
                "
            -- foo:

            \\-- yo:
            body ho"
            ),
            super::to_string(&[
                ftd::ftd2021::p1::Section::with_name("foo").and_body("-- yo:\nbody ho")
            ]),
        );

        assert_eq!(
            indoc!(
                "
            -- foo:

            --- bar:"
            ),
            super::to_string(&[ftd::ftd2021::p1::Section::with_name("foo")
                .add_sub_section(ftd::ftd2021::p1::SubSection::with_name("bar"))]),
        );
    }
}
