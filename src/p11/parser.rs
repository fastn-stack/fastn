#[derive(Debug, Clone)]
enum ParsingStateReading {
    Section,
    Header { key: String, kind: Option<String> },
    Caption,
    Body,
    Subsection,
}

#[derive(Debug)]
pub struct State {
    line_number: usize,
    sections: Vec<ftd::p11::Section>,
    content: String,
    doc_id: String,
    state: Vec<(ftd::p11::Section, Vec<ParsingStateReading>)>,
}

impl State {
    fn next(&mut self) -> ftd::p11::Result<()> {
        use itertools::Itertools;

        self.end()?;

        if self.content.trim().is_empty() {
            let sections = self.state.iter().map(|(v, _)| v.clone()).collect_vec();
            self.state = vec![];
            self.sections.extend(sections);

            return Ok(());
        }

        if let Some((_, state)) = self.get_latest_state() {
            match state.clone() {
                ParsingStateReading::Section => {
                    self.reading_block_headers()?;
                }
                ParsingStateReading::Header { key, kind } => {
                    self.reading_header_value(key.as_str(), kind)?;
                }
                ParsingStateReading::Caption => {
                    self.reading_caption_value()?;
                }
                ParsingStateReading::Body => {
                    self.reading_body_value()?;
                }
                ParsingStateReading::Subsection => {
                    self.reading_section()?;
                }
            }
        } else {
            self.reading_section()?;
        }

        Ok(())
    }

    fn end(&mut self) -> ftd::p11::Result<()> {
        let (scan_line_number, content) = self.clean_content();
        let (start_line, rest_lines) = new_line_split(content.as_str());
        if !start_line.starts_with("-- ") {
            return Ok(());
        }
        let start_line = &start_line[2..];
        let (name, caption) =
            colon_separated_values(self.line_number + 1, start_line, self.doc_id.as_str())?;
        if is_end(name.as_str()) {
            let caption = caption.ok_or_else(|| ftd::p11::Error::ParseError {
                message: "section name not provided for `end`".to_string(),
                doc_id: self.doc_id.to_string(),
                line_number: self.line_number,
            })?;
            let mut sections = vec![];
            loop {
                let line_number = self.line_number;
                let (section, state) = if let Some(state) = self.remove_latest_state() {
                    state
                } else {
                    let section = self.remove_latest_section()?.ok_or_else(|| {
                        ftd::p11::Error::ParseError {
                            message: format!("No section found to end: {}", caption),
                            doc_id: self.doc_id.to_string(),
                            line_number: self.line_number,
                        }
                    })?;
                    sections.push(section);
                    continue;
                };
                match state {
                    ParsingStateReading::Section if caption.eq(section.name.as_str()) => {
                        sections.reverse();
                        section.sub_sections.extend(sections);
                        break;
                    }
                    ParsingStateReading::Header { key, kind }
                        if caption.eq(format!("{}.{}", section.name, key).as_str()) =>
                    {
                        sections.reverse();
                        section.headers.push(ftd::p11::Header::section(
                            line_number,
                            key.as_str(),
                            kind,
                            sections,
                        ));
                        break;
                    }
                    _ => {}
                }
            }
            self.line_number += scan_line_number + 1;
            self.content = rest_lines;
            return self.end();
        }

        Ok(())
    }

    fn clean_content(&mut self) -> (usize, String) {
        let mut valid_line_number = None;
        let new_line_content = self.content.split('\n');
        let mut scan_line_number = 0;
        for (line_number, line) in new_line_content.enumerate() {
            if valid_line(line) && !line.trim().is_empty() {
                valid_line_number = Some(line_number);
                break;
            }
            scan_line_number += 1;
        }
        (
            scan_line_number,
            content_index(self.content.as_str(), valid_line_number),
        )
    }

    fn reading_section(&mut self) -> ftd::p11::Result<()> {
        let (scan_line_number, content) = self.clean_content();
        let (start_line, rest_lines) = new_line_split(content.as_str());

        if !start_line.starts_with("-- ") && !start_line.starts_with("/-- ") {
            return Err(ftd::p11::Error::SectionNotFound {
                // TODO: context should be a few lines before and after the input
                doc_id: self.doc_id.to_string(),
                line_number: self.line_number + 1,
            });
        }

        let is_commented = start_line.starts_with("/-- ");
        let line = if is_commented {
            &start_line[3..]
        } else {
            &start_line[2..]
        };
        let (name_with_kind, caption) =
            colon_separated_values(self.line_number, line, self.doc_id.as_str())?;
        let (section_name, kind) = get_name_and_kind(name_with_kind.as_str());
        let section = ftd::p11::Section {
            name: section_name,
            kind,
            caption: caption.map(|v| ftd::p11::Header::from_caption(v.as_str(), self.line_number)),
            headers: Default::default(),
            body: None,
            sub_sections: Default::default(),
            is_commented,
            line_number: self.line_number,
            block_body: false,
        };

        self.state
            .push((section, vec![ParsingStateReading::Section]));
        self.content = rest_lines;
        self.line_number += scan_line_number + 1;
        self.reading_inline_headers()?;
        self.next()
    }

    fn reading_block_headers(&mut self) -> ftd::p11::Result<()> {
        self.end()?;
        let (scan_line_number, content) = self.clean_content();
        let (section, parsing_states) =
            self.state
                .last_mut()
                .ok_or_else(|| ftd::p11::Error::SectionNotFound {
                    doc_id: self.doc_id.to_string(),
                    line_number: self.line_number,
                })?;

        let header_not_found_next_state = if !section.block_body {
            ParsingStateReading::Body
        } else {
            ParsingStateReading::Subsection
        };

        let (start_line, rest_lines) = new_line_split(content.as_str());

        if !start_line.starts_with("-- ") && !start_line.starts_with("/-- ") {
            parsing_states.push(header_not_found_next_state);
            return self.next();
        }

        let is_commented = start_line.starts_with("/-- ");
        let line = if is_commented {
            &start_line[3..]
        } else {
            &start_line[2..]
        };

        let (name_with_kind, value) =
            colon_separated_values(self.line_number, line, self.doc_id.as_str())?;
        let (key, kind) = get_name_and_kind(name_with_kind.as_str());

        let key = if let Some(key) = key.strip_prefix(format!("{}.", section.name).as_str()) {
            key
        } else {
            parsing_states.push(header_not_found_next_state);
            return self.next();
        };

        self.line_number += scan_line_number + 1;
        self.content = rest_lines;
        section.block_body = true;

        if is_caption(key) && kind.is_none() && section.caption.is_some() {
            return Err(ftd::p11::Error::MoreThanOneCaption {
                doc_id: self.doc_id.to_string(),
                line_number: section.line_number,
            });
        }
        if let Some(value) = value {
            section.headers.push(ftd::p11::Header::kv(
                self.line_number,
                key,
                kind,
                Some(value),
            ))
        } else {
            parsing_states.push(if is_caption(key) {
                ParsingStateReading::Caption
            } else if is_body(key) {
                ParsingStateReading::Body
            } else {
                ParsingStateReading::Header {
                    key: key.to_string(),
                    kind,
                }
            });
        }
        self.next()
    }

    fn reading_header_value(
        &mut self,
        header_key: &str,
        header_kind: Option<String>,
    ) -> ftd::p11::Result<()> {
        if let Err(ftd::p11::Error::SectionNotFound { .. }) = self.reading_section() {
            let mut value = vec![];
            let mut new_line_number = None;
            let mut first_line = true;
            let split_content = self.content.as_str().split('\n');
            for (line_number, line) in split_content.enumerate() {
                if line.starts_with("-- ") || line.starts_with("/-- ") {
                    new_line_number = Some(line_number);
                    break;
                }
                self.line_number += 1;
                if !valid_line(line) {
                    continue;
                }
                if first_line {
                    if !line.trim().is_empty() {
                        return Err(ftd::p11::Error::ParseError {
                            message: format!("start section header '{}' after a newline!!", line),
                            doc_id: self.doc_id.to_string(),
                            line_number: self.line_number,
                        });
                    }
                    first_line = false;
                }
                value.push(clean_line(line));
            }
            self.content = content_index(self.content.as_str(), new_line_number);
            let doc_id = self.doc_id.to_string();
            let line_number = self.line_number;
            let section = self
                .remove_latest_state()
                .ok_or(ftd::p11::Error::SectionNotFound {
                    doc_id,
                    line_number,
                })?
                .0;
            let value = value.join("\n").trim().to_string();
            section.headers.push(ftd::p11::Header::kv(
                line_number,
                header_key,
                header_kind,
                if value.is_empty() { None } else { Some(value) },
            ));
        }
        self.next()
    }

    fn reading_caption_value(&mut self) -> ftd::p11::Result<()> {
        let mut value = vec![];
        let mut new_line_number = None;
        let mut first_line = true;
        let split_content = self.content.as_str().split('\n');
        for (line_number, line) in split_content.enumerate() {
            if line.starts_with("-- ") || line.starts_with("/-- ") {
                new_line_number = Some(line_number);
                break;
            }
            self.line_number += 1;
            if !valid_line(line) {
                continue;
            }
            if first_line {
                if !line.trim().is_empty() {
                    return Err(ftd::p11::Error::ParseError {
                        message: format!("start section caption '{}' after a newline!!", line),
                        doc_id: self.doc_id.to_string(),
                        line_number: self.line_number,
                    });
                }
                first_line = false;
            }
            value.push(clean_line(line));
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;
        let section = self
            .remove_latest_state()
            .ok_or(ftd::p11::Error::SectionNotFound {
                doc_id,
                line_number,
            })?
            .0;

        let value = value.join("\n").trim().to_string();
        section.caption = Some(ftd::p11::Header::from_caption(value.as_str(), line_number));
        self.next()
    }

    fn reading_body_value(&mut self) -> ftd::p11::Result<()> {
        let mut value = vec![];
        let mut new_line_number = None;
        let mut first_line = true;
        let split_content = self.content.as_str().split('\n');
        for (line_number, line) in split_content.enumerate() {
            if line.starts_with("-- ") || line.starts_with("/-- ") {
                new_line_number = Some(line_number);
                break;
            }
            self.line_number += 1;
            if !valid_line(line) {
                continue;
            }
            if first_line {
                if !line.trim().is_empty() {
                    return Err(ftd::p11::Error::ParseError {
                        message: format!("start section body '{}' after a newline!!", line),
                        doc_id: self.doc_id.to_string(),
                        line_number: self.line_number,
                    });
                }
                first_line = false;
            }

            value.push(clean_line(line));
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;
        let section = self
            .remove_latest_state()
            .ok_or(ftd::p11::Error::SectionNotFound {
                doc_id,
                line_number,
            })?
            .0;
        let value = value.join("\n").trim().to_string();
        if !value.is_empty() {
            section.body = Some(ftd::p11::Body::new(line_number, value.as_str()));
        }
        let (section, parsing_state) = self.state.last_mut().unwrap();
        if !section.block_body {
            parsing_state.push(ParsingStateReading::Subsection);
        }
        self.next()
    }

    fn reading_inline_headers(&mut self) -> ftd::p11::Result<()> {
        let mut headers = vec![];
        let mut new_line_number = None;
        for (line_number, line) in self.content.split('\n').enumerate() {
            if line.trim().is_empty() || line.starts_with("-- ") || line.starts_with("/-- ") {
                new_line_number = Some(line_number);
                break;
            }
            if !valid_line(line) {
                self.line_number += 1;
                continue;
            }
            let line = clean_line(line);
            if let Ok((name_with_kind, caption)) =
                colon_separated_values(self.line_number, line.as_str(), self.doc_id.as_str())
            {
                let (header_key, kind) = get_name_and_kind(name_with_kind.as_str());
                headers.push(ftd::p11::Header::kv(
                    line_number,
                    header_key.as_str(),
                    kind,
                    caption,
                ));
            } else {
                new_line_number = Some(line_number);
                break;
            }
            self.line_number += 1;
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;

        let section = self
            .mut_latest_state()
            .ok_or(ftd::p11::Error::SectionNotFound {
                doc_id,
                line_number,
            })?
            .0;
        section.headers.extend(headers);
        Ok(())
    }

    fn mut_latest_state(&mut self) -> Option<(&mut ftd::p11::Section, &mut ParsingStateReading)> {
        if let Some((section, state)) = self.state.last_mut() {
            if let Some(state) = state.last_mut() {
                return Some((section, state));
            }
        }
        None
    }

    fn get_latest_state(&self) -> Option<(&ftd::p11::Section, &ParsingStateReading)> {
        if let Some((section, state)) = self.state.last() {
            if let Some(state) = state.last() {
                return Some((section, state));
            }
        }
        None
    }

    fn remove_latest_section(&mut self) -> ftd::p11::Result<Option<ftd::p11::Section>> {
        if let Some((section, state)) = self.state.last() {
            if !state.is_empty() {
                return Err(ftd::p11::Error::ParseError {
                    message: format!("`{}` section state is not yet empty", section.name),
                    doc_id: self.doc_id.to_string(),
                    line_number: self.line_number,
                });
            }
        }
        Ok(self.state.pop().map(|v| v.0))
    }

    fn remove_latest_state(&mut self) -> Option<(&mut ftd::p11::Section, ParsingStateReading)> {
        if let Some((section, state)) = self.state.last_mut() {
            if let Some(state) = state.pop() {
                return Some((section, state));
            }
        }
        None
    }
}

pub fn parse(content: &str, doc_id: &str) -> ftd::p11::Result<Vec<ftd::p11::Section>> {
    let mut state = State {
        content: content.to_string(),
        doc_id: doc_id.to_string(),
        line_number: 0,
        sections: Default::default(),
        state: Default::default(),
    };
    state.next()?;
    Ok(state.sections)
}

fn colon_separated_values(
    line_number: usize,
    line: &str,
    doc_id: &str,
) -> ftd::p11::Result<(String, Option<String>)> {
    if !line.contains(':') {
        return Err(ftd::p11::Error::ParseError {
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

fn get_name_and_kind(name_with_kind: &str) -> (String, Option<String>) {
    if let Some((kind, name)) = name_with_kind.rsplit_once(' ') {
        return (name.to_string(), Some(kind.to_string()));
    }

    (name_with_kind.to_string(), None)
}

fn clean_line(line: &str) -> String {
    if line.starts_with("\\;;") || line.starts_with("\\-- ") {
        return line[1..].to_string();
    }
    line.to_string()
}

fn valid_line(line: &str) -> bool {
    !line.starts_with(";;")
}

fn is_caption(s: &str) -> bool {
    s.eq("caption")
}

fn is_body(s: &str) -> bool {
    s.eq("body")
}

fn is_end(s: &str) -> bool {
    s.eq("end")
}

fn new_line_split(s: &str) -> (String, String) {
    if let Some((start_line, rest_lines)) = s.trim().split_once('\n') {
        (start_line.to_string(), rest_lines.to_string())
    } else {
        (s.to_string(), "".to_string())
    }
}

fn content_index(content: &str, line_number: Option<usize>) -> String {
    use itertools::Itertools;

    let new_line_content = content.split('\n');
    let content = new_line_content.collect_vec();
    match line_number {
        Some(line_number) if content.len() > line_number => content[line_number..].join("\n"),
        _ => "".to_string(),
    }
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
                    .collect::<Vec<ftd::p11::Section>>(),
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
            "-- foo:\n\n-- bar:\n\n-- end: foo",
            ftd::p11::Section::with_name("foo")
                .add_sub_section(ftd::p11::Section::with_name("bar"))
                .list()
        );

        p!(
            "-- foo: hello\n-- bar:\n\n-- end: foo",
            ftd::p11::Section::with_name("foo")
                .and_caption("hello")
                .add_sub_section(ftd::p11::Section::with_name("bar"))
                .list()
        );

        p!(
            "-- foo:\nk:v\n-- bar:\n\n-- end: foo",
            ftd::p11::Section::with_name("foo")
                .add_header_str("k", "v")
                .add_sub_section(ftd::p11::Section::with_name("bar"))
                .list()
        );

        p!(
            "-- foo:\n\nhello world\n-- bar:\n\n-- end: foo",
            ftd::p11::Section::with_name("foo")
                .and_body("hello world")
                .add_sub_section(ftd::p11::Section::with_name("bar"))
                .list()
        );

        p!(
            indoc!(
                "
            -- foo:

            body ho

            -- dodo:

            -- end: foo


            -- bar:

            bar body
            "
            ),
            vec![
                ftd::p11::Section::with_name("foo")
                    .and_body("body ho")
                    .add_sub_section(ftd::p11::Section::with_name("dodo")),
                ftd::p11::Section::with_name("bar").and_body("bar body")
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            body ho


            -- bar:

            bar body

            -- dodo:

            -- end: bar
            "
            ),
            vec![
                ftd::p11::Section::with_name("foo").and_body("body ho"),
                ftd::p11::Section::with_name("bar")
                    .and_body("bar body")
                    .add_sub_section(ftd::p11::Section::with_name("dodo"))
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            body ho


            -- bar:

            bar body

            -- dodo:
            -- rat:

            -- end: bar
            "
            ),
            vec![
                ftd::p11::Section::with_name("foo").and_body("body ho"),
                ftd::p11::Section::with_name("bar")
                    .and_body("bar body")
                    .add_sub_section(ftd::p11::Section::with_name("dodo"))
                    .add_sub_section(ftd::p11::Section::with_name("rat"))
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            body ho


            -- bar:

            -- bar.cat:

            bar body

            -- dodo:
            -- rat:

            -- end: bar
            "
            ),
            vec![
                ftd::p11::Section::with_name("foo").and_body("body ho"),
                ftd::p11::Section::with_name("bar")
                    .add_header_str("cat", "bar body")
                    .add_sub_section(ftd::p11::Section::with_name("dodo"))
                    .add_sub_section(ftd::p11::Section::with_name("rat"))
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            body ho

            -- bar:

            bar body

            -- dodo:

            hello

            -- end: bar
            "
            ),
            vec![
                ftd::p11::Section::with_name("foo").and_body("body ho"),
                ftd::p11::Section::with_name("bar")
                    .and_body("bar body")
                    .add_sub_section(ftd::p11::Section::with_name("dodo").and_body("hello"))
            ],
        );

        p!(
            "-- foo:\n\nhello world\n-- bar:\n\n-- end: foo",
            ftd::p11::Section::with_name("foo")
                .and_body("hello world")
                .add_sub_section(ftd::p11::Section::with_name("bar"))
                .list()
        );

        p!(
            "-- foo:\n\nhello world\n-- bar: foo\n\n-- end: foo",
            ftd::p11::Section::with_name("foo")
                .and_body("hello world")
                .add_sub_section(ftd::p11::Section::with_name("bar").and_caption("foo"))
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

            -- realm.rr.activity:
            okind:
            oid:
            ekind:

            null

            -- end: step

        "
            ),
            vec![ftd::p11::Section::with_name("step")
                .add_header_str("method", "GET")
                .add_sub_section(
                    ftd::p11::Section::with_name("realm.rr.activity")
                        .add_header_str("okind", "")
                        .add_header_str("oid", "")
                        .add_header_str("ekind", "")
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
            \\-- foo: bar
        "
            ),
            ftd::p11::Section::with_name("hello")
                .and_body("-- yo: whats up?\n-- foo: bar")
                .list()
        )
    }

    #[test]
    fn comments() {
        p!(
            indoc!(
                "
            ;; yo
            -- foo:
            ;; yo
            key: value

            body ho
            ;; yo

            -- bar:
            ;; yo
            b: ba
            ;; yo

            bar body
            ;; yo
            -- dodo:
            ;; yo
            k: v
            ;; yo

            hello
            ;; yo
            -- end: bar
            "
            ),
            vec![
                ftd::p11::Section::with_name("foo")
                    .and_body("body ho")
                    .add_header_str("key", "value"),
                ftd::p11::Section::with_name("bar")
                    .and_body("bar body")
                    .add_header_str("b", "ba")
                    .add_sub_section(
                        ftd::p11::Section::with_name("dodo")
                            .add_header_str("k", "v")
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
            -- dodo:
            k: v

            hello
            -- end: bar
            "
            ),
            vec![
                ftd::p11::Section::with_name("foo")
                    .and_body("body ho")
                    .add_header_str("key", "value"),
                ftd::p11::Section::with_name("bar")
                    .and_body("bar body")
                    .add_header_str("b", "ba")
                    .add_sub_section(
                        ftd::p11::Section::with_name("dodo")
                            .add_header_str("k", "v")
                            .and_body("hello")
                    )
            ],
        );
    }

    #[test]
    fn empty_key() {
        p!(
            "-- foo:\nkey: \n",
            ftd::p11::Section::with_name("foo")
                .add_header_str("key", "")
                .list()
        );

        p!(
            "-- foo:\n-- bar:\nkey:\n\n\n-- end: foo",
            ftd::p11::Section::with_name("foo")
                .add_sub_section(ftd::p11::Section::with_name("bar").add_header_str("key", ""))
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
            ftd::p11::Section::with_name("hello")
                .and_body("hello -- world: yo")
                .list()
        );

        p!(
            indoc!(
                r#"
            -- hello:

            -- realm.rr.step.body:

            {
              "body": "-- h0: Hello World\n\n-- markup:\n\ndemo cr 1\n",
              "kind": "content",
              "track": "amitu/index",
              "version": "2020-11-16T04:13:14.642892+00:00"
            }
            
            -- end: hello
        "#
            ),
            ftd::p11::Section::with_name("hello")
                .add_sub_section(ftd::p11::Section::with_name("realm.rr.step.body").and_body(
                    &indoc!(
                        r#"
                        {
                          "body": "-- h0: Hello World\n\n-- markup:\n\ndemo cr 1\n",
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
                 -- markup:

                 hello world is

                     not enough

                     lol
            "
            ),
            ftd::p11::Section::with_name("markup")
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
                ftd::p11::Section::with_name("foo").and_body("  body ho\n\nyo"),
                ftd::p11::Section::with_name("bar").and_body("    bar body")
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
            vec![ftd::p11::Section::with_name("foo").and_body("hello"),],
        );

        p!(
            indoc!(
                "
            -- foo:
            -- bar:




            hello









            -- end: foo
            "
            ),
            vec![ftd::p11::Section::with_name("foo")
                .add_sub_section(ftd::p11::Section::with_name("bar").and_body("hello"))],
        );
    }

    #[test]
    fn basic() {
        p!(
            "-- foo: bar",
            ftd::p11::Section::with_name("foo")
                .and_caption("bar")
                .list()
        );

        p!("-- foo:", ftd::p11::Section::with_name("foo").list());

        p!("-- foo: ", ftd::p11::Section::with_name("foo").list());

        p!(
            "-- foo:\nkey: value",
            ftd::p11::Section::with_name("foo")
                .add_header_str("key", "value")
                .list()
        );

        p!(
            "-- foo:\nkey: value\nk2:v2",
            ftd::p11::Section::with_name("foo")
                .add_header_str("key", "value")
                .add_header_str("k2", "v2")
                .list()
        );

        p!(
            "-- foo:\n\nbody ho",
            ftd::p11::Section::with_name("foo")
                .and_body("body ho")
                .list()
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
                ftd::p11::Section::with_name("foo").and_body("body ho"),
                ftd::p11::Section::with_name("bar").and_body("bar body")
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
                ftd::p11::Section::with_name("foo").and_body("body ho\n\nyo"),
                ftd::p11::Section::with_name("bar").and_body("bar body")
            ],
        );

        p!(
            indoc!(
                "
            -- foo:

            hello
            "
            ),
            vec![ftd::p11::Section::with_name("foo").and_body("hello"),],
        );

        f!("invalid", "foo:1 -> SectionNotFound")
    }

    #[test]
    fn strict_body() {
        // section body without headers
        f!(
            indoc!(
                "-- some-section:
                This is body
                "
            ),
            "foo:2 -> start section body 'This is body' after a newline!!"
        );

        // section body with headers
        f!(
            indoc!(
                "-- some-section:
                h1: v1
                This is body
                "
            ),
            "foo:3 -> start section body 'This is body' after a newline!!"
        );

        // subsection body without headers
        f!(
            indoc!(
                "-- some-section:
                h1: val

                -- some-sub-section:
                This is body

                -- end: some-section
                "
            ),
            "foo:5 -> start section body 'This is body' after a newline!!"
        );

        // subsection body with headers
        f!(
            indoc!(
                "-- some-section:
                h1: val

                -- some-sub-section:
                h2: val
                h3: val
                This is body

                -- end: some-section
                "
            ),
            "foo:7 -> start section body 'This is body' after a newline!!"
        );
    }

    #[test]
    fn header_section() {
        p!(
            indoc!(
                "
            -- foo:

            -- foo.bar:

            -- section:
            k1: v1

            -- section.k2:

            This is value of section k2

            -- end: foo.bar

            -- foo.body:

            bar body
            "
            ),
            ftd::p11::Section::with_name("foo")
                .and_body("bar body")
                .add_header_section(
                    "bar",
                    None,
                    ftd::p11::Section::with_name("section")
                        .add_header_str("k1", "v1")
                        .add_header_str("k2", "This is value of section k2")
                        .list()
                )
                .list(),
        );
    }

    #[test]
    fn kind() {
        p!(
            indoc!(
                "
            -- moo foo:

            -- too foo.bar:

            -- section:
            k1: v1

            -- section.k2:

            This is value of section k2

            -- end: foo.bar

            -- foo.body:

            bar body

            -- foo.caption:

            bar caption

            -- subsection:

            -- sub-subsection:
            
            This is sub-subsection

            -- end: subsection

            -- end: foo
            "
            ),
            ftd::p11::Section::with_name("foo")
                .kind("moo")
                .and_body("bar body")
                .and_caption("bar caption")
                .add_header_section(
                    "bar",
                    Some("too".to_string()),
                    ftd::p11::Section::with_name("section")
                        .add_header_str("k1", "v1")
                        .add_header_str("k2", "This is value of section k2")
                        .list()
                )
                .add_sub_section(
                    ftd::p11::Section::with_name("subsection").add_sub_section(
                        ftd::p11::Section::with_name("sub-subsection")
                            .and_body("This is sub-subsection")
                    )
                )
                .list(),
        );

        p!(
            indoc!(
                "
            -- moo foo:

            -- foo.caption:

            bar caption

            -- too foo.bar:

            -- section:
            k1: v1

            -- section.k2:

            This is value of section k2

            -- end: foo.bar

            -- foo.body:

            bar body

            -- subsection:

            -- sub-subsection:
            
            This is sub-subsection

            -- end: subsection

            -- end: foo
            "
            ),
            ftd::p11::Section::with_name("foo")
                .kind("moo")
                .and_body("bar body")
                .and_caption("bar caption")
                .add_header_section(
                    "bar",
                    Some("too".to_string()),
                    ftd::p11::Section::with_name("section")
                        .add_header_str("k1", "v1")
                        .add_header_str("k2", "This is value of section k2")
                        .list()
                )
                .add_sub_section(
                    ftd::p11::Section::with_name("subsection").add_sub_section(
                        ftd::p11::Section::with_name("sub-subsection")
                            .and_body("This is sub-subsection")
                    )
                )
                .list(),
        );

        p!(
            indoc!(
                "
            -- moo foo:

            -- foo.caption:

            bar caption

            -- foo.body:

            bar body

            -- too foo.bar:

            -- section:
            k1: v1

            -- section.k2:

            This is value of section k2

            -- end: foo.bar


            -- subsection:

            -- sub-subsection:
            
            This is sub-subsection

            -- end: subsection

            -- end: foo
            "
            ),
            ftd::p11::Section::with_name("foo")
                .kind("moo")
                .and_body("bar body")
                .and_caption("bar caption")
                .add_header_section(
                    "bar",
                    Some("too".to_string()),
                    ftd::p11::Section::with_name("section")
                        .add_header_str("k1", "v1")
                        .add_header_str("k2", "This is value of section k2")
                        .list()
                )
                .add_sub_section(
                    ftd::p11::Section::with_name("subsection").add_sub_section(
                        ftd::p11::Section::with_name("sub-subsection")
                            .and_body("This is sub-subsection")
                    )
                )
                .list(),
        );
    }
}
