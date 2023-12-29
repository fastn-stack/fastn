#[derive(Debug, Clone)]
enum ParsingStateReading {
    Section,
    Header {
        key: String,
        caption: Option<String>,
        kind: Option<String>,
        condition: Option<String>,
        line_number: usize,
    },
    Caption,
    Body,
    Subsection,
}

#[derive(Debug)]
pub struct State {
    line_number: i32,
    sections: Vec<ftd::p1::Section>,
    content: String,
    doc_id: String,
    state: Vec<(ftd::p1::Section, Vec<ParsingStateReading>)>,
}

impl State {
    fn next(&mut self) -> ftd::p1::Result<()> {
        use itertools::Itertools;

        self.reading_section()?;

        while let Some((_, mut state)) = self.get_latest_state() {
            let mut change_state = None;
            self.end(&mut change_state)?;

            if self.content.trim().is_empty() {
                let sections = self.state.iter().map(|(v, _)| v.clone()).collect_vec();
                self.state = vec![];
                self.sections.extend(sections);

                continue;
            }

            if let Some(change_state) = change_state {
                state = change_state;
            }

            match state {
                ParsingStateReading::Section => {
                    self.reading_block_headers()?;
                }
                ParsingStateReading::Header {
                    key,
                    kind,
                    condition,
                    caption,
                    line_number,
                } => {
                    self.reading_header_value(key.as_str(), caption, kind, condition, line_number)?;
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
        }

        Ok(())
    }

    fn end(&mut self, change_state: &mut Option<ParsingStateReading>) -> ftd::p1::Result<()> {
        let (scan_line_number, content) = self.clean_content();
        let (start_line, rest_lines) = new_line_split(content.as_str());
        if !start_line.starts_with("-- ") {
            return Ok(());
        }
        let start_line = &start_line[2..];
        let (name, caption) = colon_separated_values(
            ftd::p1::utils::i32_to_usize(self.line_number + 1),
            start_line,
            self.doc_id.as_str(),
        )?;
        if is_end(name.as_str()) {
            let caption = caption.ok_or_else(|| ftd::p1::Error::ParseError {
                message: "section name not provided for `end`".to_string(),
                doc_id: self.doc_id.to_string(),
                line_number: ftd::p1::utils::i32_to_usize(self.line_number),
            })?;
            let mut sections = vec![];
            loop {
                let line_number = self.line_number;
                let (section, state) = if let Some(state) = self.remove_latest_state() {
                    state
                } else {
                    let section = self.remove_latest_section()?.ok_or_else(|| {
                        ftd::p1::Error::ParseError {
                            message: format!("No section found to end: {}", caption),
                            doc_id: self.doc_id.to_string(),
                            line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                        }
                    })?;
                    sections.push(section);
                    continue;
                };
                match state {
                    ParsingStateReading::Section if caption.eq(section.name.as_str()) => {
                        sections.reverse();
                        section.sub_sections.extend(sections);
                        *change_state = None;
                        break;
                    }
                    ParsingStateReading::Header {
                        key,
                        kind,
                        condition,
                        ..
                    } if caption.eq(format!("{}.{}", section.name, key).as_str()) => {
                        sections.reverse();
                        section.headers.push(ftd::p1::Header::section(
                            ftd::p1::utils::i32_to_usize(line_number),
                            key.as_str(),
                            kind,
                            sections,
                            condition,
                        ));
                        *change_state = Some(ParsingStateReading::Section);
                        break;
                    }
                    _ => {}
                }
            }
            self.line_number += (scan_line_number as i32) + 1;
            self.content = rest_lines;
            return self.end(change_state);
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

    fn reading_section(&mut self) -> ftd::p1::Result<()> {
        use itertools::Itertools;

        let (scan_line_number, content) = self.clean_content();
        let (start_line, rest_lines) = new_line_split(content.as_str());

        if !start_line.starts_with("-- ") && !start_line.starts_with("/-- ") {
            return if start_line.is_empty() {
                Ok(())
            } else {
                Err(ftd::p1::Error::SectionNotFound {
                    // TODO: context should be a few lines before and after the input
                    doc_id: self.doc_id.to_string(),
                    line_number: ftd::p1::utils::i32_to_usize(
                        self.line_number + (scan_line_number as i32) + 1,
                    ),
                })
            };
        }

        let start_line = clean_line_with_trim(start_line.as_str());

        let is_commented = start_line.starts_with("/-- ");
        let line = if is_commented {
            &start_line[3..]
        } else {
            &start_line[2..]
        };

        let (name_with_kind, caption) =
        //  section-kind section-name: caption
            colon_separated_values(ftd::p1::utils::i32_to_usize(self.line_number), line, self
                .doc_id.as_str())?;
        let (section_name, kind) = get_name_and_kind(name_with_kind.as_str());
        let last_section = self.get_latest_state().map(|v| v.0);
        match last_section {
            Some(section) if section_name.starts_with(format!("{}.", section.name).as_str()) => {
                let module_headers = section
                    .headers
                    .0
                    .iter()
                    .filter(|h| h.is_module_kind())
                    .collect_vec();
                let found_module = module_headers.iter().find(|h| {
                    h.is_module_kind()
                        && section_name
                            .strip_prefix(format!("{}.", section.name).as_str())
                            .unwrap_or(section_name.as_str())
                            .starts_with(h.get_key().as_str())
                });

                if found_module.is_none() {
                    return Err(ftd::p1::Error::SectionNotFound {
                        doc_id: self.doc_id.to_string(),
                        line_number: ftd::p1::utils::i32_to_usize(
                            self.line_number + (scan_line_number as i32) + 1,
                        ),
                    });
                }
            }
            _ => {}
        }

        self.line_number += (scan_line_number as i32) + 1;
        let section = ftd::p1::Section {
            name: section_name,
            kind,
            caption: caption.map(|v| {
                ftd::p1::Header::from_caption(
                    v.as_str(),
                    ftd::p1::utils::i32_to_usize(self.line_number),
                )
            }),
            headers: Default::default(),
            body: None,
            sub_sections: Default::default(),
            is_commented,
            line_number: ftd::p1::utils::i32_to_usize(self.line_number),
            block_body: false,
        };

        self.state
            .push((section, vec![ParsingStateReading::Section]));
        self.content = rest_lines;
        self.reading_inline_headers()?;
        Ok(())
    }

    fn eval_from_kv_header(
        header_key: &str,
        header_data: HeaderData,
        section: &mut ftd::p1::Section,
        doc_id: &str,
    ) -> ftd::p1::Result<()> {
        if let Some((header, field)) = header_key.split_once('.') {
            // Record Field syntax
            if let Ok(existing_header) =
                section
                    .headers
                    .find_once_mut(header, doc_id, header_data.line_number)
            {
                // Existing header found with same name
                match existing_header {
                    ftd::p1::Header::BlockRecordHeader(br_header) => {
                        // Existing header is of block record type
                        // So update its fields
                        let current_field = ftd::p1::Header::kv(
                            header_data.line_number,
                            field,
                            header_data.kind,
                            header_data.value,
                            header_data.condition,
                            header_data.source,
                        );
                        br_header.fields.push(current_field);
                    }
                    ftd::p1::Header::KV(kv) => {
                        // Existing header is of KV type
                        let mut existing_header_caption = None;
                        let mut existing_header_body = (None, None);

                        match kv.source {
                            ftd::p1::header::KVSource::Caption => {
                                existing_header_caption = kv.value.to_owned()
                            }
                            ftd::p1::header::KVSource::Body => {
                                existing_header_body = (kv.value.to_owned(), Some(kv.line_number))
                            }
                            _ => unimplemented!(),
                        }

                        let block_record_header = ftd::p1::Header::block_record_header(
                            header,
                            kv.kind.to_owned(),
                            existing_header_caption,
                            existing_header_body,
                            vec![ftd::p1::Header::kv(
                                header_data.line_number,
                                field,
                                header_data.kind,
                                header_data.value,
                                header_data.condition,
                                header_data.source,
                            )],
                            kv.condition.to_owned(),
                            kv.line_number,
                        );
                        *existing_header = block_record_header;
                    }
                    _ => unimplemented!(),
                }
            } else {
                // No existing block record header found under section.headers
                section.headers.push(ftd::p1::Header::block_record_header(
                    header,
                    header_data.kind.clone(),
                    None,
                    (None, None),
                    vec![ftd::p1::Header::kv(
                        header_data.line_number,
                        field,
                        header_data.kind,
                        header_data.value,
                        header_data.condition.clone(),
                        header_data.source,
                    )],
                    header_data.condition,
                    header_data.line_number,
                ));
            }
        } else {
            // Normal header
            section.headers.push(ftd::p1::Header::kv(
                header_data.line_number,
                header_key,
                header_data.kind,
                header_data.value,
                header_data.condition,
                header_data.source,
            ));
        }
        Ok(())
    }

    fn reading_block_headers(&mut self) -> ftd::p1::Result<()> {
        use itertools::Itertools;

        self.end(&mut None)?;
        let (scan_line_number, content) = self.clean_content();
        let (section, parsing_states) =
            self.state
                .last_mut()
                .ok_or_else(|| ftd::p1::Error::SectionNotFound {
                    doc_id: self.doc_id.to_string(),
                    line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                })?;

        let header_not_found_next_state = if !section.block_body {
            ParsingStateReading::Body
        } else {
            ParsingStateReading::Subsection
        };

        let (start_line, rest_lines) = new_line_split(content.as_str());

        if !start_line.starts_with("-- ") && !start_line.starts_with("/-- ") {
            parsing_states.push(header_not_found_next_state);
            return Ok(());
        }

        let is_commented = start_line.starts_with("/-- ");
        let line = if is_commented {
            &start_line[3..]
        } else {
            &start_line[2..]
        };

        let (name_with_kind, value) = colon_separated_values(
            ftd::p1::utils::i32_to_usize(self.line_number),
            line,
            self.doc_id.as_str(),
        )?;
        let (key, kind) = get_name_and_kind(name_with_kind.as_str());

        let module_headers = section
            .headers
            .0
            .iter()
            .filter(|h| h.is_module_kind())
            .collect_vec();
        if let Some(possible_module) =
            key.strip_prefix(format!("{}.", section.name.as_str()).as_str())
        {
            for m in module_headers.iter() {
                if possible_module.starts_with(m.get_key().as_str()) {
                    parsing_states.push(header_not_found_next_state);
                    return Ok(());
                }
            }
        }

        let key = if let Some(key) = key.strip_prefix(format!("{}.", section.name).as_str()) {
            key
        } else {
            parsing_states.push(header_not_found_next_state);
            return Ok(());
        };

        self.line_number += (scan_line_number as i32) + 1;
        self.content = rest_lines;
        section.block_body = true;

        let condition = get_block_header_condition(
            &mut self.content,
            &mut self.line_number,
            self.doc_id.as_str(),
        )?;

        if is_caption(key) && kind.is_none() && section.caption.is_some() {
            return Err(ftd::p1::Error::MoreThanOneCaption {
                doc_id: self.doc_id.to_string(),
                line_number: section.line_number,
            });
        }

        let doc_id = self.doc_id.clone();
        let (next_line, _) = new_line_split(self.content.as_str());
        let next_inline_header = next_line.contains(':') && !next_line.starts_with("-- ");

        if let (Some(value), true) = (value.clone(), !next_inline_header) {
            let header_data = HeaderData::new(
                Some(value),
                kind,
                condition,
                Some(ftd::p1::header::KVSource::Caption),
                ftd::p1::utils::i32_to_usize(self.line_number),
            );
            Self::eval_from_kv_header(key, header_data, section, doc_id.as_str())?;
        } else {
            parsing_states.push(if is_caption(key) {
                ParsingStateReading::Caption
            } else if is_body(key) {
                ParsingStateReading::Body
            } else {
                ParsingStateReading::Header {
                    key: key.to_string(),
                    caption: value,
                    kind,
                    condition,
                    line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                }
            });
        }
        Ok(())
    }

    fn reading_header_value(
        &mut self,
        header_key: &str,
        header_caption: Option<String>,
        header_kind: Option<String>,
        header_condition: Option<String>,
        header_line_number: usize,
    ) -> ftd::p1::Result<()> {
        if let Err(ftd::p1::Error::SectionNotFound { .. }) = self.reading_section() {
            let mut value: (Vec<String>, Option<usize>) = (vec![], None);
            let mut inline_record_headers: ftd::Map<HeaderData> = ftd::Map::new();
            let mut reading_value = false;
            let mut new_line_number = None;
            let mut first_line = true;
            let split_content = self.content.as_str().split('\n');
            for (line_number, line) in split_content.enumerate() {
                let trimmed_line = line.trim_start();
                if trimmed_line.starts_with("-- ") || trimmed_line.starts_with("/-- ") {
                    new_line_number = Some(line_number);
                    break;
                }
                self.line_number += 1;
                if !valid_line(line) {
                    continue;
                }
                let inline_record_header_found = trimmed_line.contains(':')
                    && !trimmed_line.starts_with('\\')
                    && !trimmed_line.starts_with(";;");
                if first_line {
                    if !trimmed_line.is_empty() && !inline_record_header_found {
                        return Err(ftd::p1::Error::ParseError {
                            message: format!("start section body '{}' after a newline!!", line),
                            doc_id: self.doc_id.to_string(),
                            line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                        });
                    }
                    first_line = false;
                }

                if inline_record_header_found && !reading_value {
                    if let Ok((name_with_kind, caption)) = colon_separated_values(
                        ftd::p1::utils::i32_to_usize(self.line_number),
                        line,
                        self.doc_id.as_str(),
                    ) {
                        // Caption, kind, condition, line_number
                        let (header_key, kind, condition) =
                            get_name_kind_and_condition(name_with_kind.as_str());
                        inline_record_headers.insert(
                            header_key,
                            HeaderData::new(
                                caption,
                                kind,
                                condition,
                                Some(Default::default()),
                                ftd::p1::utils::i32_to_usize(self.line_number),
                            ),
                        );
                    }
                } else if !trimmed_line.is_empty() || !value.0.is_empty() {
                    // value(body) = (vec![string], line_number)
                    reading_value = true;
                    value.0.push(clean_line(line));
                    if value.1.is_none() {
                        value.1 = Some(ftd::p1::utils::i32_to_usize(self.line_number));
                    }
                }
            }
            self.content = content_index(self.content.as_str(), new_line_number);
            let doc_id = self.doc_id.to_string();
            let _line_number = self.line_number;
            let section = self
                .remove_latest_state()
                .ok_or(ftd::p1::Error::SectionNotFound {
                    doc_id: doc_id.clone(),
                    line_number: header_line_number,
                })?
                .0;
            let value = (trim_body(value.0.join("\n").as_str()).to_string(), value.1);
            if !inline_record_headers.is_empty()
                || (header_caption.is_some() && !value.0.is_empty())
            {
                let fields = inline_record_headers
                    .iter()
                    .map(|(key, data)| {
                        ftd::p1::Header::kv(
                            data.line_number,
                            key,
                            data.kind.to_owned(),
                            data.value.to_owned(),
                            data.condition.to_owned(),
                            data.source.to_owned(),
                        )
                    })
                    .collect();
                section.headers.push(ftd::p1::Header::block_record_header(
                    header_key,
                    header_kind,
                    header_caption,
                    if value.0.is_empty() {
                        (None, None)
                    } else {
                        (Some(value.0), value.1)
                    },
                    fields,
                    header_condition,
                    header_line_number,
                ));
            } else {
                let header_data = HeaderData {
                    value: if !value.0.is_empty() {
                        Some(value.0)
                    } else {
                        None
                    },
                    kind: header_kind,
                    condition: header_condition,
                    source: Some(ftd::p1::header::KVSource::Body),
                    line_number: value.1.unwrap_or(header_line_number),
                };
                Self::eval_from_kv_header(header_key, header_data, section, doc_id.as_str())?;
            }
        }
        Ok(())
    }

    fn reading_caption_value(&mut self) -> ftd::p1::Result<()> {
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
                    return Err(ftd::p1::Error::ParseError {
                        message: format!("start section caption '{}' after a newline!!", line),
                        doc_id: self.doc_id.to_string(),
                        line_number: ftd::p1::utils::i32_to_usize(self.line_number),
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
            .ok_or(ftd::p1::Error::SectionNotFound {
                doc_id,
                line_number: ftd::p1::utils::i32_to_usize(line_number),
            })?
            .0;

        let value = value.join("\n").trim().to_string();
        section.caption = Some(ftd::p1::Header::from_caption(
            value.as_str(),
            ftd::p1::utils::i32_to_usize(line_number),
        ));
        Ok(())
    }

    fn reading_body_value(&mut self) -> ftd::p1::Result<()> {
        let mut value = vec![];
        let mut new_line_number = None;
        let mut first_line = true;
        let split_content = self.content.as_str().split('\n');
        for (line_number, line) in split_content.enumerate() {
            if line.trim_start().starts_with("-- ") || line.trim_start().starts_with("/-- ") {
                new_line_number = Some(line_number);
                break;
            }
            self.line_number += 1;
            if !valid_line(line) {
                continue;
            }
            if first_line {
                if !line.trim().is_empty() {
                    return Err(ftd::p1::Error::ParseError {
                        message: format!("start section body '{}' after a newline!!", line),
                        doc_id: self.doc_id.to_string(),
                        line_number: ftd::p1::utils::i32_to_usize(self.line_number),
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
            .ok_or(ftd::p1::Error::SectionNotFound {
                doc_id,
                line_number: ftd::p1::utils::i32_to_usize(line_number),
            })?
            .0;
        let value = value.join("\n").to_string();
        if !value.trim().is_empty() {
            section.body = Some(ftd::p1::Body::new(
                ftd::p1::utils::i32_to_usize(line_number),
                trim_body(value.as_str()).as_str(),
            ));
        }
        let (section, parsing_state) = self.state.last_mut().unwrap();
        if !section.block_body {
            parsing_state.push(ParsingStateReading::Subsection);
        }
        Ok(())
    }

    // There should not be no new line in the headers
    fn reading_inline_headers(&mut self) -> ftd::p1::Result<()> {
        let mut headers = vec![];
        let mut new_line_number = None;
        for (line_number, mut line) in self.content.split('\n').enumerate() {
            line = line.trim_start();
            if line.is_empty() || line.starts_with("-- ") || line.starts_with("/-- ") {
                new_line_number = Some(line_number);
                break;
            }
            if !valid_line(line) {
                self.line_number += 1;
                continue;
            }
            let line = clean_line_with_trim(line);
            if let Ok((name_with_kind, caption)) = colon_separated_values(
                ftd::p1::utils::i32_to_usize(self.line_number),
                line.as_str(),
                self.doc_id.as_str(),
            ) {
                let (header_key, kind, condition) =
                    get_name_kind_and_condition(name_with_kind.as_str());
                self.line_number += 1;
                headers.push(ftd::p1::Header::kv(
                    ftd::p1::utils::i32_to_usize(self.line_number),
                    header_key.as_str(),
                    kind,
                    caption,
                    condition,
                    Some(ftd::p1::header::KVSource::Header),
                ));
            } else {
                new_line_number = Some(line_number);
                break;
            }
        }
        self.content = content_index(self.content.as_str(), new_line_number);
        let doc_id = self.doc_id.to_string();
        let line_number = self.line_number;

        let section = self
            .mut_latest_state()
            .ok_or(ftd::p1::Error::SectionNotFound {
                doc_id,
                line_number: ftd::p1::utils::i32_to_usize(line_number),
            })?
            .0;
        section.headers.0.extend(headers);
        Ok(())
    }

    fn mut_latest_state(&mut self) -> Option<(&mut ftd::p1::Section, &mut ParsingStateReading)> {
        if let Some((section, state)) = self.state.last_mut() {
            if let Some(state) = state.last_mut() {
                return Some((section, state));
            }
        }
        None
    }

    fn get_latest_state(&self) -> Option<(ftd::p1::Section, ParsingStateReading)> {
        if let Some((section, state)) = self.state.last() {
            if let Some(state) = state.last() {
                return Some((section.to_owned(), state.to_owned()));
            }
        }
        None
    }

    fn remove_latest_section(&mut self) -> ftd::p1::Result<Option<ftd::p1::Section>> {
        if let Some((section, state)) = self.state.last() {
            if !state.is_empty() {
                return Err(ftd::p1::Error::ParseError {
                    message: format!("`{}` section state is not yet empty", section.name),
                    doc_id: self.doc_id.to_string(),
                    line_number: ftd::p1::utils::i32_to_usize(self.line_number),
                });
            }
        }
        Ok(self.state.pop().map(|v| v.0))
    }

    fn remove_latest_state(&mut self) -> Option<(&mut ftd::p1::Section, ParsingStateReading)> {
        if let Some((section, state)) = self.state.last_mut() {
            if let Some(state) = state.pop() {
                return Some((section, state));
            }
        }
        None
    }
}

#[derive(Debug)]
pub struct HeaderData {
    value: Option<String>,
    kind: Option<String>,
    condition: Option<String>,
    source: Option<ftd::p1::header::KVSource>,
    line_number: usize,
}

impl HeaderData {
    pub fn new(
        value: Option<String>,
        kind: Option<String>,
        condition: Option<String>,
        source: Option<ftd::p1::header::KVSource>,
        line_number: usize,
    ) -> Self {
        HeaderData {
            value,
            kind,
            condition,
            source,
            line_number,
        }
    }
}

pub fn parse(content: &str, doc_id: &str) -> ftd::p1::Result<Vec<ftd::p1::Section>> {
    parse_with_line_number(content, doc_id, 0)
}

pub fn parse_with_line_number(
    content: &str,
    doc_id: &str,
    line_number: usize,
) -> ftd::p1::Result<Vec<ftd::p1::Section>> {
    let mut state = State {
        content: content.to_string(),
        doc_id: doc_id.to_string(),
        line_number: if line_number > 0 {
            -(line_number as i32)
        } else {
            0
        },
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
) -> ftd::p1::Result<(String, Option<String>)> {
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

fn get_name_and_kind(name_with_kind: &str) -> (String, Option<String>) {
    let mut name_with_kind = name_with_kind.to_owned();

    // Fix spacing for functional parameters inside parenthesis (if user provides)
    if let (Some(si), Some(ei)) = (name_with_kind.find('('), name_with_kind.find(')')) {
        if si < ei {
            // All Content before start ( bracket
            let before_brackets = &name_with_kind[..si];
            // All content after start ( bracket and all inner content excluding ) bracket
            let mut bracket_content_and_beyond = name_with_kind[si..ei].replace(' ', "");
            // Push any remaining characters including ) and after end bracket
            bracket_content_and_beyond.push_str(&name_with_kind[ei..]);
            name_with_kind = format!("{}{}", before_brackets, bracket_content_and_beyond);
        }
    }

    if let Some((kind, name)) = name_with_kind.rsplit_once(' ') {
        return (name.to_string(), Some(kind.to_string()));
    }

    (name_with_kind.to_string(), None)
}

fn get_name_kind_and_condition(name_with_kind: &str) -> (String, Option<String>, Option<String>) {
    let (name_with_kind, condition) = if let Some((name_with_kind, condition)) =
        name_with_kind.split_once(ftd::p1::utils::INLINE_IF)
    {
        (name_with_kind.to_string(), Some(condition.to_string()))
    } else {
        (name_with_kind.to_string(), None)
    };
    if let Some((kind, name)) = name_with_kind.rsplit_once(' ') {
        return (name.to_string(), Some(kind.to_string()), condition);
    }

    (name_with_kind, None, condition)
}

fn clean_line(line: &str) -> String {
    let trimmed_line = line.trim_start();
    if trimmed_line.starts_with("\\;;") || trimmed_line.starts_with("\\-- ") {
        return format!(
            "{}{}",
            " ".repeat(line.len() - trimmed_line.len()),
            &trimmed_line[1..]
        );
    }

    if !line.contains("<hl>") {
        return remove_inline_comments(line);
    }

    format!(
        "{}{}",
        " ".repeat(line.len() - trimmed_line.len()),
        trimmed_line
    )
}

fn clean_line_with_trim(line: &str) -> String {
    clean_line(line).trim_start().to_string()
}

fn trim_body(s: &str) -> String {
    let mut leading_spaces_count = usize::MAX;
    let mut value = vec![];

    // Get minimum number of the starting space in the whole body, ignoring empty line
    for line in s.split('\n') {
        let trimmed_line = line.trim_start().to_string();
        let current_leading_spaces_count = line.len() - trimmed_line.len();
        if !line.is_empty() && current_leading_spaces_count < leading_spaces_count {
            leading_spaces_count = current_leading_spaces_count;
        }
    }
    if leading_spaces_count == usize::MAX {
        leading_spaces_count = 0;
    }

    // Trim the lines of the body upto the leading_spaces_count
    for line in s.split('\n') {
        let mut trimmed_line = line.trim_start().to_string();
        let current_leading_spaces_count = line.len() - trimmed_line.len();
        if current_leading_spaces_count > leading_spaces_count {
            trimmed_line = format!(
                "{}{}",
                " ".repeat(current_leading_spaces_count - leading_spaces_count),
                trimmed_line
            );
        }
        value.push(trimmed_line);
    }
    value.join("\n")
}

fn remove_inline_comments(line: &str) -> String {
    let mut output = String::new();
    let mut chars = line.chars().peekable();
    let mut escape = false;
    let mut count = 0;

    while let Some(c) = chars.next() {
        if c.eq(&'\\') {
            if !escape {
                escape = true;
            }

            count += 1;

            if let Some(nc) = chars.peek() {
                if nc.eq(&';') {
                    output.push(';');
                    chars.next();
                    continue;
                } else if nc.ne(&'\\') {
                    escape = false;
                    count = 0;
                }
            }
        }

        if c.eq(&';') {
            if escape {
                if count % 2 == 0 {
                    output.pop();
                    break;
                } else {
                    escape = false;
                    count = 0;
                }
            } else if let Some(nc) = chars.peek() {
                if nc.eq(&';') {
                    break;
                }
            }
        }

        if escape {
            escape = false;
            count = 0;
        }

        output.push(c);
    }

    output.to_string()
}

fn valid_line(line: &str) -> bool {
    !line.trim().starts_with(";;")
}

fn is_caption(s: &str) -> bool {
    s.contains("caption")
}

fn is_body(s: &str) -> bool {
    s.eq("body")
}

fn is_end(s: &str) -> bool {
    s.eq("end")
}

fn new_line_split(s: &str) -> (String, String) {
    if let Some((start_line, rest_lines)) = s.trim().split_once('\n') {
        (start_line.trim_start().to_string(), rest_lines.to_string())
    } else {
        (s.trim_start().to_string(), "".to_string())
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

pub(crate) fn get_block_header_condition(
    content: &mut String,
    line_number: &mut i32,
    doc_id: &str,
) -> ftd::p1::Result<Option<String>> {
    let mut condition = None;
    let mut new_line_number = None;
    for (line_number, line) in content.split('\n').enumerate() {
        if !valid_line(line) {
            continue;
        }
        let line = clean_line_with_trim(line);
        if let Ok((name_with_kind, caption)) =
            colon_separated_values(line_number, line.as_str(), doc_id)
        {
            if name_with_kind.eq(ftd::p1::utils::IF) {
                condition = caption;
                new_line_number = Some(line_number + 1);
            }
        }
        break;
    }

    if let Some(new_line_number) = new_line_number {
        *content = content_index(content.as_str(), Some(new_line_number));
        *line_number += new_line_number as i32;
    }

    Ok(condition)
}
