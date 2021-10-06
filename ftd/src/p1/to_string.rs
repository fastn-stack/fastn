pub fn to_string(p1: &[crate::p1::Section]) -> String {
    p1.iter()
        .map(|v| v.to_string().trim().to_string())
        .collect::<Vec<String>>()
        .join("\n\n\n")
}

impl std::fmt::Display for crate::p1::Section {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "-- {}:", self.name.as_str())?;
        if let Some(ref caption) = self.caption {
            write!(f, " {}", caption)?;
        }

        for (k, v) in self.header.0.iter() {
            write!(f, "\n{}: {}", k, v)?;
        }

        writeln!(f)?;

        if let Some(ref body) = self.body {
            write!(f, "\n{}\n", escape_body(body))?;
        }

        for sub in self.sub_sections.0.iter() {
            write!(f, "\n{}", sub)?;
        }

        Ok(())
    }
}

impl std::fmt::Display for crate::p1::SubSection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "--- {}:", self.name.as_str())?;
        if let Some(ref caption) = self.caption {
            write!(f, " {}", caption)?;
        }

        for (k, v) in self.header.0.iter() {
            write!(f, "\n{}: {}", k, v)?;
        }

        if let Some(ref body) = self.body {
            write!(f, "\n\n{}", escape_body(body))?;
        }

        writeln!(f)
    }
}

fn escape_body(body: &str) -> String {
    let body = "\n".to_string() + body;
    let body = body.replace("\n-- ", "\n\\-- ");
    body.replace("\n--- ", "\n\\--- ").trim().to_string()
}

#[cfg(test)]
mod test {
    use {indoc::indoc, pretty_assertions::assert_eq}; // macro

    #[test]
    pub fn subsection_formatter() {
        let s = indoc!(
            "
            -- ftd.row:

            --- ftd.text:


            hello world
            "
        );
        let sections = ftd::p1::parse(s).expect("Cannot parse to section");
        assert_eq!(
            indoc!(
                "-- ftd.row:

                --- ftd.text:

                hello world"
            ),
            super::to_string(&sections)
        )
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
                crate::p1::Section::with_name("foo")
                    .and_body("body ho")
                    .add_header("key", "value")
                    .add_sub_section(
                        crate::p1::SubSection::with_name("dodo")
                            .and_caption("foo")
                            .add_header("foo", "bar"),
                    )
                    .add_sub_section(
                        crate::p1::SubSection::with_name("dodo").add_header("foo", "bar")
                    ),
                crate::p1::Section::with_name("bar").and_body("bar body")
            ]),
        );

        assert_eq!(
            indoc!(
                "
            -- foo:

            \\-- yo:
            body ho"
            ),
            super::to_string(&vec![
                crate::p1::Section::with_name("foo").and_body("-- yo:\nbody ho")
            ]),
        );

        assert_eq!(
            indoc!(
                "
            -- foo:

            --- bar:"
            ),
            super::to_string(&vec![crate::p1::Section::with_name("foo")
                .add_sub_section(crate::p1::SubSection::with_name("bar")),]),
        );
    }
}
