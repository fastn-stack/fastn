use itertools::Itertools;

#[derive(Debug, Clone)]
enum ParsingState {
    ReadingSection,
    ReadingHeader { key: String, kind: Option<String> },
    ReadingCaption,
    ReadingBody,
    ReadingSubsection,
}

#[derive(Debug, Default)]
pub struct State {
    line_number: usize,
    sections: Vec<ftd::p11::Section>,
    content: String,
    doc_id: String,
    state: Vec<(ftd::p11::Section, Vec<ParsingState>)>,
}

impl State {
    fn parse_(&mut self) -> ftd::p11::Result<()> {
        self.content = self.clean_content();
        while self.content.is_empty() {
            self.reading_section()?;
            self.content = self.clean_content();
        }
        Ok(())
    }

    fn next(&mut self) -> ftd::p11::Result<()> {
        self.content = self.clean_content();
        if self.content.trim().is_empty() {
            let sections = self.state.iter().map(|(v, _)| v.clone()).collect_vec();
            self.state = vec![];
            self.sections.extend(sections);

            return Ok(());
        }
        self.end()?;

        if let Some((_, state)) = self.get_latest_state() {
            match state.clone() {
                ParsingState::ReadingSection => {
                    self.reading_block_headers()?;
                }
                ParsingState::ReadingHeader { key, kind } => {
                    self.reading_header_value(key.as_str(), kind.to_owned())?;
                }
                ParsingState::ReadingCaption => {
                    self.reading_caption_value()?;
                }
                ParsingState::ReadingBody => {
                    self.reading_body_value()?;
                }
                ParsingState::ReadingSubsection => {
                    self.reading_section()?;
                }
            }
        } else {
            self.reading_section()?;
        }
        Ok(())
    }

    fn end(&mut self) -> ftd::p11::Result<()> {
        let (start_line, reset_lines) = new_line_split(self.content.as_str());
        if !start_line.starts_with("-- ") {
            return Ok(());
        }
        let start_line = &start_line[2..];
        let (name, caption) =
            colon_separated_values(self.line_number + 1, start_line, self.doc_id.as_str())?;
        if is_end(name.as_str()) {
            self.line_number += 1;
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
                    ParsingState::ReadingSection if caption.eq(section.name.as_str()) => {
                        section.sub_sections.extend(sections);
                        break;
                    }
                    ParsingState::ReadingHeader { key, kind }
                        if caption.eq(format!("{}.{}", section.name, key).as_str()) =>
                    {
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
            self.content = reset_lines;
        }

        Ok(())
    }

    fn clean_content(&mut self) -> String {
        let mut valid_line_number = 0;
        let new_line_content = self.content.split('\n');
        for (line_number, line) in new_line_content.enumerate() {
            self.line_number += 1;
            if !valid_line(line) {
                continue;
            }
            valid_line_number = line_number;
            break;
        }
        content_index(self.content.as_str(), valid_line_number)
    }

    fn reading_section(&mut self) -> ftd::p11::Result<()> {
        self.content = self.clean_content();
        let (start_line, rest_lines) = new_line_split(self.content.as_str());
        self.line_number += 1;

        if !start_line.starts_with("-- ") && !start_line.starts_with("/-- ") {
            return Err(ftd::p11::Error::SectionNotFound {
                // TODO: context should be a few lines before and after the input
                doc_id: self.doc_id.to_string(),
                line_number: self.line_number,
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
            .push((section, vec![ParsingState::ReadingSection]));
        self.content = rest_lines;
        self.reading_inline_headers()?;
        self.next()
    }

    fn reading_block_headers(&mut self) -> ftd::p11::Result<()> {
        self.content = self.clean_content();
        self.end()?;
        let (section, parsing_states) =
            self.state
                .last_mut()
                .ok_or_else(|| ftd::p11::Error::SectionNotFound {
                    doc_id: self.doc_id.to_string(),
                    line_number: self.line_number,
                })?;

        let header_not_found_next_state = if !section.block_body {
            ParsingState::ReadingBody
        } else {
            ParsingState::ReadingSubsection
        };

        let (start_line, rest_lines) = new_line_split(self.content.as_str());

        if !start_line.starts_with("-- ") && !start_line.starts_with("/-- ") {
            parsing_states.push(header_not_found_next_state);
            return self.next();
        }

        self.line_number += 1;
        self.content = rest_lines;
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

        section.block_body = true;

        if is_caption(key) && kind.is_none() {
            if section.caption.is_some() {
                return Err(ftd::p11::Error::MoreThanOneCaption {
                    doc_id: self.doc_id.to_string(),
                    line_number: section.line_number,
                });
            }
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
                ParsingState::ReadingCaption
            } else if is_body(key) {
                ParsingState::ReadingBody
            } else {
                ParsingState::ReadingHeader {
                    key: "".to_string(),
                    kind,
                }
            });
        }
        Ok(())
    }

    fn reading_header_value(
        &mut self,
        header_key: &str,
        header_kind: Option<String>,
    ) -> ftd::p11::Result<()> {
        self.content = self.clean_content();
        if let Err(ftd::p11::Error::SectionNotFound { .. }) = self.reading_section() {
            let mut value = vec![];
            let mut new_line_number = 0;
            let split_content = self.content.as_str().split('\n');
            for (line_number, line) in split_content.enumerate() {
                new_line_number = line_number;
                if line.starts_with("-- ") || line.starts_with("/-- ") {
                    break;
                }
                value.push(line.to_string());
                self.line_number += 1;
            }
            self.content = content_index(self.content.as_str(), new_line_number);
            let doc_id = self.doc_id.to_string();
            let line_number = self.line_number;
            let section = self
                .remove_latest_state()
                .ok_or_else(|| ftd::p11::Error::SectionNotFound {
                    doc_id,
                    line_number,
                })?
                .0;
            section.headers.push(ftd::p11::Header::kv(
                line_number,
                header_key,
                header_kind,
                Some(value.join("\n")),
            ));
        }
        self.next()
    }

    fn reading_caption_value(&mut self) -> ftd::p11::Result<()> {
        self.content = self.clean_content();
        let mut value = vec![];
        let mut new_line_number = 0;
        let split_content = self.content.as_str().split('\n');
        for (line_number, line) in split_content.enumerate() {
            new_line_number = line_number;
            if line.starts_with("-- ") || line.starts_with("/-- ") {
                break;
            }
            value.push(line.to_string());
            self.line_number += 1;
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;
        let section = self
            .remove_latest_state()
            .ok_or_else(|| ftd::p11::Error::SectionNotFound {
                doc_id,
                line_number,
            })?
            .0;
        section.caption = Some(ftd::p11::Header::from_caption(
            &value.join("\n"),
            line_number,
        ));
        self.next()
    }

    fn reading_body_value(&mut self) -> ftd::p11::Result<()> {
        self.content = self.clean_content();
        let mut value = vec![];
        let mut new_line_number = 0;
        let split_content = self.content.as_str().split('\n');
        for (line_number, line) in split_content.enumerate() {
            new_line_number = line_number;
            if line.starts_with("-- ") || line.starts_with("/-- ") {
                break;
            }
            value.push(line.to_string());
            self.line_number += 1;
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;
        let section = self
            .remove_latest_state()
            .ok_or_else(|| ftd::p11::Error::SectionNotFound {
                doc_id,
                line_number,
            })?
            .0;
        section.body = Some(ftd::p11::Body::body(line_number, &value.join("\n")));
        self.next()
    }

    fn reading_inline_headers(&mut self) -> ftd::p11::Result<()> {
        self.content = self.clean_content();
        let mut headers = vec![];
        for (line_number, line) in self.content.split('\n').enumerate() {
            self.line_number += 1;
            if !valid_line(line) {
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
                break;
            }
        }

        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;

        let section = self
            .mut_latest_state()
            .ok_or_else(|| ftd::p11::Error::SectionNotFound {
                doc_id,
                line_number,
            })?
            .0;
        section.headers.extend(headers);
        Ok(())
    }

    fn mut_latest_state(&mut self) -> Option<(&mut ftd::p11::Section, &mut ParsingState)> {
        if let Some((section, state)) = self.state.last_mut() {
            if let Some(state) = state.last_mut() {
                return Some((section, state));
            }
        }
        None
    }

    fn get_latest_state(&self) -> Option<(&ftd::p11::Section, &ParsingState)> {
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

    fn remove_latest_state(&mut self) -> Option<(&mut ftd::p11::Section, ParsingState)> {
        if let Some((section, state)) = self.state.last_mut() {
            if let Some(state) = state.pop() {
                return Some((section, state));
            }
        }
        None
    }
}

pub fn parse(content: &str, doc_id: &str) -> ftd::p11::Result<Vec<ftd::p11::Section>> {
    let mut state = State::default();
    state.content = content.to_string();
    state.doc_id = doc_id.to_string();
    state.parse_()?;
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
    if let Some((name, kind)) = name_with_kind.rsplit_once(' ') {
        return (name.to_string(), Some(kind.to_string()));
    }

    (name_with_kind.to_string(), None)
}

fn clean_line(line: &str) -> String {
    if line.starts_with("\\;;") {
        return line[1..].to_string();
    }
    line.to_string()
}

fn valid_line(line: &str) -> bool {
    !(line.starts_with(";;") || line.trim().is_empty())
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

fn content_index(content: &str, line_number: usize) -> String {
    let new_line_content = content.split('\n');
    let content = new_line_content.collect_vec();
    if content.len().eq(&(line_number + 1)) {
        return "".to_string();
    }
    content[line_number..].join("\n")
}
