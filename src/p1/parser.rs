pub use ftd::p1::{Error, Header, Result, Section, SubSection, SubSections};

#[derive(Debug)]
enum ParsingState {
    WaitingForSection,
    ReadingHeader,
    ReadingBody,
    ReadingSubsectionHeader,
    ReadingSubSectionBody,
}

#[derive(Debug)]
pub struct State {
    state: ParsingState,
    section: Option<Section>,
    sub_section: Option<SubSection>,
    sections: Vec<Section>,
}

fn colon_separated_values(
    line_number: usize,
    line: &str,
    doc_id: &str,
) -> Result<(String, Option<String>)> {
    if !line.contains(':') {
        return Err(ftd::p1::Error::ParseError {
            message: format!(": is missing in: {}", line),
            // TODO: context should be a few lines before and after the input
            doc_id: doc_id.to_string(),
            line_number,
        });
    }

    let mut parts = line.splitn(2, ':');
    let name = parts.next().unwrap().trim().to_string();

    let caption = match parts.next() {
        Some(c) if c.trim().is_empty() => None,
        Some(c) => Some(c.trim().to_string()),
        None => None,
    };

    Ok((name, caption))
}

fn to_body(b: Option<(usize, String)>) -> Option<(usize, String)> {
    match b {
        Some(b) if b.1.trim().is_empty() => None,
        Some(b) => Some((b.0, b.1.trim_end().to_string())),
        None => None,
    }
}

impl State {
    fn waiting_for_section(&mut self, line_number: usize, line: &str, doc_id: &str) -> Result<()> {
        if line.trim().is_empty() {
            return Ok(());
        }

        let is_commented = line.starts_with("/-- ");

        if !line.starts_with("-- ") && !line.starts_with("/-- ") {
            return Err(ftd::p1::Error::ParseError {
                message: format!("Expecting -- , found: {}", line,),
                // TODO: context should be a few lines before and after the input
                doc_id: doc_id.to_string(),
                line_number,
            });
        }

        if let Some(mut s) = self.section.take() {
            if let Some(mut sub) = self.sub_section.take() {
                sub.body = to_body(sub.body.take());
                s.sub_sections.0.push(sub)
            }

            s.body = to_body(s.body.take());
            self.sections.push(s);
        }

        let line = if is_commented { &line[3..] } else { &line[2..] };
        let (name, caption) = colon_separated_values(line_number, line, doc_id)?;

        self.section = Some(Section {
            name,
            caption,
            header: Default::default(),
            body: None,
            sub_sections: Default::default(),
            is_commented,
            line_number,
        });

        self.state = ParsingState::ReadingHeader;

        Ok(())
    }

    fn reading_header(&mut self, line_number: usize, line: &str, doc_id: &str) -> Result<()> {
        // change state to reading body iff after an empty line is found
        if line.trim().is_empty() {
            self.state = ParsingState::ReadingBody;
            return Ok(());
        }

        if line.starts_with("-- ") || line.starts_with("/-- ") {
            return self.waiting_for_section(line_number, line, doc_id);
        }

        if line.starts_with("--- ") || line.starts_with("/--- ") {
            return self.read_subsection(line_number, line, doc_id);
        }

        // If no empty line or start of next section/subsection found
        // immediately after reading all possible headers for the current section/subsection
        // then throw error
        if !line.contains(':') {
            return Err(ftd::p1::Error::ParseError {
                message: "start section body after a newline!!".to_string(),
                doc_id: doc_id.to_string(),
                line_number,
            });
        }

        let (name, value) = colon_separated_values(line_number, line, doc_id)?;
        if let Some(mut s) = self.section.take() {
            s.header.add(
                &line_number,
                name.as_str(),
                value.unwrap_or_else(|| "".to_string()).as_str(),
            );
            self.section = Some(s);
        }

        Ok(())
    }

    fn reading_sub_header(&mut self, line_number: usize, line: &str, doc_id: &str) -> Result<()> {
        let line = line.trim();
        if line.trim().is_empty() {
            self.state = ParsingState::ReadingSubSectionBody;
            return Ok(());
        }
        if line.starts_with("-- ") || line.starts_with("/-- ") {
            return self.waiting_for_section(line_number, line, doc_id);
        }
        if line.starts_with("--- ") || line.starts_with("/--- ") {
            return self.read_subsection(line_number, line, doc_id);
        }

        // similar strict check for subsection, change state to reading body
        // iff an empty line is found prior to reading body
        // or read next section/subsection
        // otherwise throw error
        if !line.contains(':') {
            return Err(ftd::p1::Error::ParseError {
                message: "start sub-section body after a newline!!".to_string(),
                doc_id: doc_id.to_string(),
                line_number,
            });
        }
        let (name, value) = colon_separated_values(line_number, line, doc_id)?;
        if let Some(mut s) = self.sub_section.take() {
            s.header.add(
                &line_number,
                name.as_str(),
                value.unwrap_or_else(|| "".to_string()).as_str(),
            );
            self.sub_section = Some(s);
        }

        Ok(())
    }

    fn reading_body(&mut self, line_number: usize, line: &str, doc_id: &str) -> Result<()> {
        self.state = ParsingState::ReadingBody;

        if line.starts_with("-- ") || line.starts_with("/-- ") {
            return self.waiting_for_section(line_number, line, doc_id);
        }

        if line.starts_with("--- ") || line.starts_with("/--- ") {
            return self.read_subsection(line_number, line, doc_id);
        }

        let line = if line.starts_with("\\-- ") || line.starts_with("\\--- ") {
            &line[1..]
        } else {
            line
        };

        if let Some(mut s) = self.section.take() {
            // empty lines at the beginning are ignore
            if line.trim().is_empty() && s.body.as_ref().map(|v| v.1.is_empty()).unwrap_or(true) {
                self.section = Some(s);
                return Ok(());
            }

            s.body = Some(match s.body {
                Some(ref b) => (b.0.to_owned(), b.1.to_string() + line + "\n"),
                None => (line_number, line.to_string() + "\n"),
            });
            self.section = Some(s);
        }

        Ok(())
    }

    fn reading_sub_body(&mut self, line_number: usize, line: &str, doc_id: &str) -> Result<()> {
        self.state = ParsingState::ReadingSubSectionBody;

        if line.starts_with("-- ") || line.starts_with("/-- ") {
            return self.waiting_for_section(line_number, line, doc_id);
        }

        if line.starts_with("--- ") || line.starts_with("/--- ") {
            return self.read_subsection(line_number, line, doc_id);
        }

        let line = if line.starts_with("\\-- ") || line.starts_with("\\--- ") {
            &line[1..]
        } else {
            line
        };

        if let Some(mut s) = self.sub_section.take() {
            if line.trim().is_empty() && s.body.as_ref().map(|v| v.1.is_empty()).unwrap_or(true) {
                self.sub_section = Some(s);
                return Ok(());
            }

            s.body = Some(match s.body {
                Some(ref b) => (b.0.to_owned(), b.1.to_string() + line + "\n"),
                None => (line_number, line.to_string() + "\n"),
            });
            self.sub_section = Some(s);
        }

        Ok(())
    }

    fn read_subsection(&mut self, line_number: usize, line: &str, doc_id: &str) -> Result<()> {
        if let Some(mut sub) = self.sub_section.take() {
            sub.body = to_body(sub.body.take());
            if let Some(mut s) = self.section.take() {
                s.sub_sections.0.push(sub);
                self.section = Some(s);
            }
        };

        let is_commented = line.starts_with("/--- ");

        let line = if is_commented { &line[4..] } else { &line[3..] };
        let (name, caption) = colon_separated_values(line_number, line, doc_id)?;

        self.sub_section = Some(SubSection {
            name,
            caption,
            header: Default::default(),
            body: None,
            is_commented,
            line_number,
        });

        self.state = ParsingState::ReadingSubsectionHeader;

        Ok(())
    }

    fn finalize(mut self) -> Vec<Section> {
        if let Some(mut s) = self.section.take() {
            if let Some(mut sub) = self.sub_section.take() {
                sub.body = to_body(sub.body.take());
                s.sub_sections.0.push(sub)
            }
            s.body = to_body(s.body.take());
            self.sections.push(s)
        } else if self.sub_section.is_some() {
            unreachable!("subsection without section!")
        };

        self.sections
    }
}

pub fn parse(s: &str, doc_id: &str) -> Result<Vec<Section>> {
    let mut state = State {
        state: ParsingState::WaitingForSection,
        section: None,
        sub_section: None,
        sections: vec![],
    };

    for (line_number, mut line) in s.split('\n').enumerate() {
        let line_number = line_number + 1;
        if line.starts_with(';') {
            continue;
        }
        if line.starts_with("\\;") {
            line = &line[1..];
        }
        match state.state {
            ParsingState::WaitingForSection => {
                state.waiting_for_section(line_number, line, doc_id)?
            }
            ParsingState::ReadingHeader => state.reading_header(line_number, line, doc_id)?,
            ParsingState::ReadingBody => state.reading_body(line_number, line, doc_id)?,
            ParsingState::ReadingSubsectionHeader => {
                state.reading_sub_header(line_number, line, doc_id)?
            }
            ParsingState::ReadingSubSectionBody => {
                state.reading_sub_body(line_number, line, doc_id)?
            }
        }
    }

    Ok(state.finalize())
}

#[cfg(test)]
mod test {
    use {indoc::indoc, pretty_assertions::assert_eq}; // macro

    // these are macros instead of functions so stack trace top points to actual
    // invocation of these, instead of inside these, so jumping to failing test
    // is easier.
    macro_rules! p {
        ($s:expr, $t: expr,) => {
            p!($s, $t)
        };
        ($s:expr, $t: expr) => {
            assert_eq!(
                super::parse($s, "foo")
                    .unwrap_or_else(|e| panic!("{}", e))
                    .iter()
                    .map(|v| v.without_line_number())
                    .collect::<Vec<ftd::p1::Section>>(),
                $t
            )
        };
    }

    macro_rules! f {
        ($s:expr, $m: expr,) => {
            f!($s, $m)
        };
        ($s:expr, $m: expr) => {
            match super::parse($s, "foo") {
                Ok(r) => panic!("expected failure, found: {:?}", r),
                Err(e) => {
                    let expected = $m.trim();
                    let f2 = e.to_string();
                    let found = f2.trim();
                    if expected != found {
                        let patch = diffy::create_patch(expected, found);
                        let f = diffy::PatchFormatter::new().with_color();
                        print!(
                            "{}",
                            f.fmt_patch(&patch)
                                .to_string()
                                .replace("\\ No newline at end of file", "")
                        );
                        println!("expected:\n{}\nfound:\n{}\n", expected, f2);
                        panic!("test failed")
                    }
                }
            }
        };
    }

    #[test]
    fn sub_section() {
        p!(
            "-- foo:\n\n--- bar:",
            super::Section::with_name("foo")
                .add_sub_section(super::SubSection::with_name("bar"))
                .list()
        );

        p!(
            "-- foo: hello\n--- bar:",
            super::Section::with_name("foo")
                .and_caption("hello")
                .add_sub_section(super::SubSection::with_name("bar"))
                .list()
        );

        p!(
            "-- foo:\nk:v\n--- bar:",
            super::Section::with_name("foo")
                .add_header("k", "v")
                .add_sub_section(super::SubSection::with_name("bar"))
                .list()
        );

        p!(
            "-- foo:\nhello world\n--- bar:",
            super::Section::with_name("foo")
                .and_body("hello world")
                .add_sub_section(super::SubSection::with_name("bar"))
                .list()
        );

        p!(
            indoc!(
                "
            -- foo:
            body ho
            --- dodo:
            -- bar:
            bar body
            "
            ),
            vec![
                super::Section::with_name("foo")
                    .and_body("body ho")
                    .add_sub_section(super::SubSection::with_name("dodo")),
                super::Section::with_name("bar").and_body("bar body")
            ],
        );

        p!(
            indoc!(
                "
            -- foo:
            body ho
            -- bar:
            bar body
            --- dodo:
            "
            ),
            vec![
                super::Section::with_name("foo").and_body("body ho"),
                super::Section::with_name("bar")
                    .and_body("bar body")
                    .add_sub_section(super::SubSection::with_name("dodo"))
            ],
        );

        p!(
            indoc!(
                "
            -- foo:
            body ho
            -- bar:
            bar body
            --- dodo:
            --- rat:
            "
            ),
            vec![
                super::Section::with_name("foo").and_body("body ho"),
                super::Section::with_name("bar")
                    .and_body("bar body")
                    .add_sub_section(super::SubSection::with_name("dodo"))
                    .add_sub_section(super::SubSection::with_name("rat"))
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            body ho

            -- bar:

            bar body

            --- dodo:
            --- rat:
            "
            ),
            vec![
                super::Section::with_name("foo").and_body("body ho"),
                super::Section::with_name("bar")
                    .and_body("bar body")
                    .add_sub_section(super::SubSection::with_name("dodo"))
                    .add_sub_section(super::SubSection::with_name("rat"))
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            body ho

            -- bar:

            bar body

            --- dodo:

            --- rat:


            "
            ),
            vec![
                super::Section::with_name("foo").and_body("body ho"),
                super::Section::with_name("bar")
                    .and_body("bar body")
                    .add_sub_section(super::SubSection::with_name("dodo"))
                    .add_sub_section(super::SubSection::with_name("rat"))
            ],
        );

        p!(
            indoc!(
                "
            -- foo:
            body ho
            -- bar:
            bar body
            --- dodo:
            hello
            "
            ),
            vec![
                super::Section::with_name("foo").and_body("body ho"),
                super::Section::with_name("bar")
                    .and_body("bar body")
                    .add_sub_section(super::SubSection::with_name("dodo").and_body("hello"))
            ],
        );

        p!(
            "-- foo:\nhello world\n--- bar:",
            super::Section::with_name("foo")
                .and_body("hello world")
                .add_sub_section(super::SubSection::with_name("bar"))
                .list()
        );

        p!(
            "-- foo:\nhello world\n--- bar: foo",
            super::Section::with_name("foo")
                .and_body("hello world")
                .add_sub_section(super::SubSection::with_name("bar").and_caption("foo"))
                .list()
        );
    }

    #[test]
    fn activity() {
        p!(
            indoc!(
                "
            -- step:
            method: GET

            --- realm.rr.activity:
            okind:
            oid:
            ekind:

            null

        "
            ),
            vec![super::Section::with_name("step")
                .add_header("method", "GET")
                .add_sub_section(
                    super::SubSection::with_name("realm.rr.activity")
                        .add_header("okind", "")
                        .add_header("oid", "")
                        .add_header("ekind", "")
                        .and_body("null")
                )]
        )
    }

    #[test]
    fn escaping() {
        p!(
            indoc!(
                "
            -- hello:

            \\-- yo: whats up?
            \\--- foo: bar
        "
            ),
            super::Section::with_name("hello")
                .and_body("-- yo: whats up?\n--- foo: bar")
                .list()
        )
    }

    #[test]
    fn comments() {
        p!(
            indoc!(
                "
            ; yo
            -- foo:
            ; yo
            key: value

            body ho
            ; yo

            -- bar:
            ; yo
            b: ba
            ; yo
            bar body
            ; yo
            --- dodo:
            ; yo
            k: v
            ; yo
            hello
            ; yo
            "
            ),
            vec![
                super::Section::with_name("foo")
                    .and_body("body ho")
                    .add_header("key", "value"),
                super::Section::with_name("bar")
                    .and_body("bar body")
                    .add_header("b", "ba")
                    .add_sub_section(
                        super::SubSection::with_name("dodo")
                            .add_header("k", "v")
                            .and_body("hello")
                    )
            ],
        );
    }

    #[test]
    fn two() {
        p!(
            indoc!(
                "
            -- foo:
            key: value

            body ho

            -- bar:
            b: ba
            bar body
            --- dodo:
            k: v
            hello
            "
            ),
            vec![
                super::Section::with_name("foo")
                    .and_body("body ho")
                    .add_header("key", "value"),
                super::Section::with_name("bar")
                    .and_body("bar body")
                    .add_header("b", "ba")
                    .add_sub_section(
                        super::SubSection::with_name("dodo")
                            .add_header("k", "v")
                            .and_body("hello")
                    )
            ],
        );
    }

    #[test]
    fn empty_key() {
        p!(
            "-- foo:\nkey: \n",
            super::Section::with_name("foo")
                .add_header("key", "")
                .list()
        );

        p!(
            "-- foo:\n--- bar:\nkey:\n",
            super::Section::with_name("foo")
                .add_sub_section(super::SubSection::with_name("bar").add_header("key", ""))
                .list()
        )
    }

    #[test]
    fn with_dash_dash() {
        p!(
            indoc!(
                r#"
            -- hello:

            hello -- world: yo
        "#
            ),
            super::Section::with_name("hello")
                .and_body("hello -- world: yo")
                .list()
        );

        p!(
            indoc!(
                r#"
            -- hello:

            --- realm.rr.step.body:

            {
              "body": "-- h0: Hello World\n\n-- markdown:\n\ndemo cr 1\n",
              "kind": "content",
              "track": "amitu/index",
              "version": "2020-11-16T04:13:14.642892+00:00"
            }
        "#
            ),
            super::Section::with_name("hello")
                .add_sub_section(super::SubSection::with_name("realm.rr.step.body").and_body(
                    &indoc!(
                        r#"
                        {
                          "body": "-- h0: Hello World\n\n-- markdown:\n\ndemo cr 1\n",
                          "kind": "content",
                          "track": "amitu/index",
                          "version": "2020-11-16T04:13:14.642892+00:00"
                        }"#
                    )
                ))
                .list()
        );
    }

    #[test]
    fn indented_body() {
        p!(
            &indoc!(
                "
                 -- markdown:
                 hello world is

                     not enough

                     lol
            "
            ),
            super::Section::with_name("markdown")
                .and_body("hello world is\n\n    not enough\n\n    lol")
                .list(),
        );
        p!(
            indoc!(
                "
            -- foo:

              body ho

            yo

            -- bar:

                bar body

            "
            ),
            vec![
                super::Section::with_name("foo").and_body("  body ho\n\nyo"),
                super::Section::with_name("bar").and_body("    bar body")
            ],
        );
    }

    #[test]
    fn body_with_empty_lines() {
        p!(
            indoc!(
                "
            -- foo:





            hello









            "
            ),
            vec![super::Section::with_name("foo").and_body("hello"),],
        );

        p!(
            indoc!(
                "
            -- foo:
            --- bar:




            hello









            "
            ),
            vec![super::Section::with_name("foo")
                .add_sub_section(super::SubSection::with_name("bar").and_body("hello"))],
        );
    }

    #[test]
    fn basic() {
        p!(
            "-- foo: bar",
            super::Section::with_name("foo").and_caption("bar").list()
        );

        p!("-- foo:", super::Section::with_name("foo").list());

        p!("-- foo: ", super::Section::with_name("foo").list());

        p!(
            "-- foo:\nkey: value",
            super::Section::with_name("foo")
                .add_header("key", "value")
                .list()
        );

        p!(
            "-- foo:\nkey: value\nk2:v2",
            super::Section::with_name("foo")
                .add_header("key", "value")
                .add_header("k2", "v2")
                .list()
        );

        p!(
            "-- foo:\nbody ho",
            super::Section::with_name("foo").and_body("body ho").list()
        );

        p!(
            indoc!(
                "
            -- foo:
            body ho
            -- bar:
            bar body
            "
            ),
            vec![
                super::Section::with_name("foo").and_body("body ho"),
                super::Section::with_name("bar").and_body("bar body")
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            body ho

            yo

            -- bar:

            bar body

            "
            ),
            vec![
                super::Section::with_name("foo").and_body("body ho\n\nyo"),
                super::Section::with_name("bar").and_body("bar body")
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            hello
            "
            ),
            vec![super::Section::with_name("foo").and_body("hello"),],
        );

        f!("invalid", "foo:1 -> Expecting -- , found: invalid")
    }
}
