#![allow(dead_code)]

/// calls `inner_ender` for all the embedded section inside section in the
/// list and then calls `ender` for the list itself
pub fn ender(
    source: &str,
    o: &mut fastn_p1::Document,
    sections: Vec<fastn_p1::Section>,
) -> Vec<fastn_p1::Section> {
    // recursive part
    let sections = sections
        .into_iter()
        .map(|s| section_ender(source, o, s))
        .collect();

    // non recursive part
    inner_ender(source, o, sections)
}

fn section_ender(
    source: &str,
    o: &mut fastn_p1::Document,
    mut section: fastn_p1::Section,
) -> fastn_p1::Section {
    if let Some(caption) = section.caption {
        section.caption = Some(header_value_ender(source, o, caption));
    }
    section.headers = section
        .headers
        .into_iter()
        .map(|mut h| {
            h.value = header_value_ender(source, o, h.value);
            h
        })
        .collect();
    if let Some(body) = section.body {
        section.body = Some(header_value_ender(source, o, body));
    }
    section.children = ender(source, o, section.children);
    section
}

fn header_value_ender(
    source: &str,
    o: &mut fastn_p1::Document,
    header: fastn_p1::HeaderValue,
) -> fastn_p1::HeaderValue {
    header
        .into_iter()
        .map(|ses| match ses {
            fastn_p1::SES::String(span) => fastn_p1::SES::String(span),
            fastn_p1::SES::Expression {
                start,
                end,
                content,
            } => fastn_p1::SES::Expression {
                start,
                end,
                content: header_value_ender(source, o, content),
            },
            fastn_p1::SES::Section(sections) => fastn_p1::SES::Section(ender(source, o, sections)),
        })
        .collect()
}

/// converts a section list, with interleaved `-- end: <section-name>`, into a nested section list
///
/// example:
/// [{section: "foo"}, {section: "bar"}, "-- end: foo"] -> [{section: "foo", children: [{section: "bar"}]}]
fn inner_ender<T: SectionProxy>(
    source: &str,
    o: &mut fastn_p1::Document,
    sections: Vec<T>,
) -> Vec<T> {
    let mut stack = Vec::new();
    'outer: for mut section in sections {
        match section.mark(source).unwrap() {
            Mark::Start(_name) => {
                stack.push(section);
            }
            Mark::End(e_name) => {
                let mut children = Vec::new();
                while let Some(candidate) = stack.pop() {
                    match candidate.mark(source).unwrap() {
                        Mark::Start(name) => {
                            if name == e_name {
                                section.add_children(children);
                                stack.push(section);
                                continue 'outer;
                            } else {
                                children.push(candidate);
                            }
                        }
                        Mark::End(_name) => unreachable!("we never put section end on the stack"),
                    }
                }
                // we have run out of sections, and we have not found the section end, return
                // error, put the children back on the stack
                o.items.push(fastn_p1::Spanned {
                    span: section.span(),
                    value: fastn_p1::Item::Error(fastn_p1::SingleError::EndWithoutStart),
                });
                stack.extend(children.into_iter().rev());
            }
        }
    }
    stack
}

enum Mark<'input> {
    Start(&'input str),
    End(&'input str),
}

/// we are using a proxy trait so we can write tests against a fake type, and then implement the
/// trait for the real Section type
trait SectionProxy: Sized {
    /// returns the name of the section, and if it starts or ends the section
    fn mark<'input>(
        &'input self,
        source: &'input str,
    ) -> Result<Mark<'input>, fastn_p1::SingleError>;
    fn add_children(&mut self, children: Vec<Self>);
    fn span(&self) -> fastn_p1::Span;
}

impl SectionProxy for fastn_p1::Section {
    fn mark<'input>(
        &'input self,
        source: &'input str,
    ) -> Result<Mark<'input>, fastn_p1::SingleError> {
        let span = &self.init.name.name.name;
        let name = &source[span.start..span.end];
        if name != "end" {
            return Ok(Mark::Start(name));
        }

        let caption = match self.caption.as_ref() {
            Some(caption) => caption,
            None => return Err(fastn_p1::SingleError::SectionNameNotFoundForEnd),
        };

        let v = match (caption.get(0), caption.len()) {
            (Some(fastn_p1::SES::String(span)), 1) => &source[span.start..span.end].trim(),
            (Some(_), _) => return Err(fastn_p1::SingleError::EndContainsData),
            (None, _) => return Err(fastn_p1::SingleError::SectionNameNotFoundForEnd),
        };

        // if v is not a single word, we have a problem
        if v.contains(' ') || v.contains('\t') {
            // SES::String cannot contain new lines.
            return Err(fastn_p1::SingleError::EndContainsData);
        }

        Ok(Mark::End(v))
    }

    fn add_children(&mut self, children: Vec<Self>) {
        self.children = children;
    }

    fn span(&self) -> fastn_p1::Span {
        self.init.dashdash.clone()
    }
}

#[cfg(test)]
mod test {
    #[allow(dead_code)] // #[expect(dead_code)] is not working
    struct DummySection {
        name: String,
        is_end: bool,
        children: Vec<DummySection>,
    }

    impl super::SectionProxy for DummySection {
        fn mark<'input>(
            &'input self,
            _source: &'input str,
        ) -> Result<super::Mark<'input>, fastn_p1::SingleError> {
            if self.is_end {
                Ok(super::Mark::End(&self.name))
            } else {
                Ok(super::Mark::Start(&self.name))
            }
        }

        fn add_children(&mut self, children: Vec<Self>) {
            self.children = children;
        }

        fn span(&self) -> fastn_p1::Span {
            Default::default()
        }
    }

    // format: foo -> bar -> /foo (
    fn parse(name: &str) -> Vec<DummySection> {
        let mut sections = vec![];
        let mut current = &mut sections;
        for part in name.split(" -> ") {
            let is_end = part.starts_with('/');
            let name = if is_end { &part[1..] } else { part };
            let section = DummySection {
                name: name.to_string(),
                is_end,
                children: vec![],
            };
            current.push(section);
            if !is_end {
                current = &mut current.last_mut().unwrap().children;
            }
        }
        sections
    }

    // foo containing bar and baz will look like this: foo [bar [], baz []]
    fn to_str(sections: &[DummySection]) -> String {
        fn to_str_(s: &mut String, sections: &[DummySection]) {
            // we are using peekable iterator so we can check if we are at the end
            let mut iterator = sections.iter().peekable();
            while let Some(section) = iterator.next() {
                s.push_str(&section.name);
                if section.children.is_empty() {
                    if iterator.peek().is_some() {
                        s.push(' ');
                    }
                    continue;
                }
                s.push_str(" [");
                if !section.children.is_empty() {
                    to_str_(s, &section.children);
                }
                s.push(']');
            }
        }

        let mut s = String::new();
        to_str_(&mut s, sections);
        s
    }

    #[track_caller]
    fn t(source: &str, expected: &str) {
        let mut o = fastn_p1::Document::default();
        let sections = parse(source);
        let sections = super::inner_ender(source, &mut o, sections);
        assert_eq!(to_str(&sections), expected);
        assert!(o.items.is_empty());
    }

    #[track_caller]
    fn f(source: &str, expected: &str, errors: Vec<fastn_p1::SingleError>) {
        let mut o = fastn_p1::Document::default();
        let sections = parse(source);
        let sections = super::inner_ender(source, &mut o, sections);
        assert_eq!(to_str(&sections), expected);

        assert_eq!(
            o.items,
            errors
                .into_iter()
                .map(|e| fastn_p1::Spanned {
                    span: Default::default(),
                    value: fastn_p1::Item::Error(e),
                })
                .collect::<Vec<_>>()
        );
    }

    #[test]
    fn test_inner_ender() {
        t("foo -> bar -> baz -> /foo", "foo [bar, baz]");
        f(
            "foo -> bar -> /baz",
            "foo, bar", // we eat the `-- end` sections even if they don't match
            vec![fastn_p1::SingleError::EndWithoutStart],
        );
        t("foo -> /foo", "foo");
        t("foo -> /foo -> bar", "foo, bar");
        t("bar -> foo -> /foo -> baz", "bar, foo, baz");
        t("bar -> a -> /a -> foo -> /foo -> baz", "bar, a, foo, baz");
        t(
            "bar -> a -> b -> /a -> foo -> /foo -> baz",
            "bar, a [b], foo, baz",
        );
        t("foo -> bar -> baz -> /bar -> /foo", "foo, [bar [baz]]");
        t(
            "foo -> bar -> baz -> a -> /bar -> /foo",
            "foo, [bar [baz, a]]",
        );
        t(
            "foo -> bar -> baz -> a -> /a -> /bar -> /foo",
            "foo [bar [baz, a]]",
        );
    }
}
