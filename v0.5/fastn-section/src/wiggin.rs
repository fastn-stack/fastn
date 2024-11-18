/// calls `inner_ender` for all the embedded section inside section in the
/// list and then calls `ender` for the list itself
#[allow(dead_code)]
pub fn ender(
    source: &str,
    o: &mut fastn_section::Document,
    sections: Vec<fastn_section::Section>,
) -> Vec<fastn_section::Section> {
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
    o: &mut fastn_section::Document,
    mut section: fastn_section::Section,
) -> fastn_section::Section {
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
    o: &mut fastn_section::Document,
    header: fastn_section::HeaderValue,
) -> fastn_section::HeaderValue {
    fastn_section::HeaderValue(
        header
            .0
            .into_iter()
            .map(|ses| match ses {
                fastn_section::Tes::Text(span) => fastn_section::Tes::Text(span),
                fastn_section::Tes::Expression {
                    start,
                    end,
                    content,
                } => fastn_section::Tes::Expression {
                    start,
                    end,
                    content: header_value_ender(source, o, content),
                },
                fastn_section::Tes::Section(sections) => {
                    fastn_section::Tes::Section(ender(source, o, sections))
                }
            })
            .collect(),
    )
}

/// converts a section list, with interleaved `-- end: <section-name>`, into a nested section list
///
/// example:
/// [{section: "foo"}, {section: "bar"}, "-- end: foo"] -> [{section: "foo", children: [{section: "bar"}]}]
fn inner_ender<T: SectionProxy>(
    source: &str,
    o: &mut fastn_section::Document,
    sections: Vec<T>,
) -> Vec<T> {
    let mut stack = Vec::new();
    'outer: for section in sections {
        match section.mark(source).unwrap() {
            // If the section is a start marker, push it onto the stack
            Mark::Start(_name) => {
                stack.push(section);
            }
            // If the section is an end marker, find the corresponding start marker in the stack
            Mark::End(e_name) => {
                let mut children = Vec::new(); // Collect children for the matching section
                while let Some(mut candidate) = stack.pop() {
                    match candidate.mark(source).unwrap() {
                        Mark::Start(name) => {
                            // If the candidate section name is the same as the end section name
                            // and is not ended, add the children to the candidate.
                            // Example:
                            // 1. -- bar:
                            // 2.   -- bar:
                            // 3.   -- end: bar
                            // 4.   -- foo:
                            // 5.   -- end: foo
                            // 6. -- end: bar
                            // When we reach `6. -- end: bar`, we will pop `5. -- foo` and
                            // `4. -- bar` and add them to the candidate. Though the `4. -- bar`
                            // section name is same as the end section name `bar`, but it is ended,
                            // so it will be considered as candidate, not potential parent. The
                            // `1. -- bar` section will be considered as a parent as it's not yet
                            // ended.
                            if name == e_name && !candidate.has_ended() {
                                candidate.add_children(children);
                                stack.push(candidate);
                                continue 'outer;
                            } else {
                                children.insert(0, candidate);
                            }
                        }
                        Mark::End(_name) => unreachable!("we never put section end on the stack"),
                    }
                }
                // we have run out of sections, and we have not found the section end, return
                // error, put the children back on the stack
                o.errors.push(fastn_section::Spanned {
                    span: section.span(),
                    value: fastn_section::Error::EndWithoutStart,
                });
                stack.extend(children.into_iter());
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
trait SectionProxy: Sized + std::fmt::Debug {
    /// returns the name of the section, and if it starts or ends the section
    fn mark<'input>(
        &'input self,
        source: &'input str,
    ) -> Result<Mark<'input>, fastn_section::Error>;

    /// Adds a list of children to the current section. It is typically called when the section
    /// is finalized or ended, hence `self.has_ended` function, if called after this, should return
    /// `true`.
    fn add_children(&mut self, children: Vec<Self>);

    /// Checks if the current section is marked as ended.
    ///
    /// # Returns
    /// - `true` if the section has been closed by an end marker.
    /// - `false` if the section is still open and can accept further nesting.
    fn has_ended(&self) -> bool;
    fn span(&self) -> fastn_section::Span;
}

impl SectionProxy for fastn_section::Section {
    fn mark<'input>(
        &'input self,
        source: &'input str,
    ) -> Result<Mark<'input>, fastn_section::Error> {
        let span = &self.init.name.name.name;
        let name = &source[span.start..span.end];
        if name != "end" {
            return Ok(Mark::Start(name));
        }

        let caption = match self.caption.as_ref() {
            Some(caption) => caption,
            None => return Err(fastn_section::Error::SectionNameNotFoundForEnd),
        };

        let v = match (caption.0.get(0), caption.0.len()) {
            (Some(fastn_section::Tes::Text(span)), 1) => &source[span.start..span.end].trim(),
            (Some(_), _) => return Err(fastn_section::Error::EndContainsData),
            (None, _) => return Err(fastn_section::Error::SectionNameNotFoundForEnd),
        };

        // if v is not a single word, we have a problem
        if v.contains(' ') || v.contains('\t') {
            // SES::String cannot contain new lines.
            return Err(fastn_section::Error::EndContainsData);
        }

        Ok(Mark::End(v))
    }

    fn add_children(&mut self, children: Vec<Self>) {
        self.children = children;

        // Since this function is called by `SectionProxy::inner_end` when end is encountered even
        // when children is empty, we can safely assume `self.has_end` is set to true regardless of
        // children being empty or not.
        self.has_end = true;
    }

    fn has_ended(&self) -> bool {
        self.has_end
    }

    fn span(&self) -> fastn_section::Span {
        self.init.dashdash.clone()
    }
}

#[cfg(test)]
mod test {
    #[allow(dead_code)] // #[expect(dead_code)] is not working
    #[derive(Debug)]
    struct DummySection {
        name: String,
        // does the section have end mark like
        // `/foo`
        // where `/` marks end of the section `foo`
        has_end_mark: bool,
        // has the section ended like
        // `foo -> /foo`
        // where `foo` has ended by `/foo`
        has_ended: bool,
        children: Vec<DummySection>,
    }

    impl super::SectionProxy for DummySection {
        fn mark<'input>(
            &'input self,
            _source: &'input str,
        ) -> Result<super::Mark<'input>, fastn_section::Error> {
            if self.has_end_mark {
                Ok(super::Mark::End(&self.name))
            } else {
                Ok(super::Mark::Start(&self.name))
            }
        }

        fn add_children(&mut self, children: Vec<Self>) {
            self.children = children;
            self.has_ended = true;
        }

        fn has_ended(&self) -> bool {
            self.has_ended
        }

        fn span(&self) -> fastn_section::Span {
            fastn_section::utils::dummy_span()
        }
    }

    // format: foo -> bar -> /foo (
    fn parse(name: &str) -> Vec<DummySection> {
        let mut sections = vec![];
        let current = &mut sections;
        for part in name.split(" -> ") {
            let is_end = part.starts_with('/');
            let name = if is_end { &part[1..] } else { part };
            let section = DummySection {
                name: name.to_string(),
                has_end_mark: is_end,
                has_ended: false,
                children: vec![],
            };
            current.push(section);
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
                        s.push_str(", ");
                    }
                    continue;
                }
                s.push_str(" [");
                if !section.children.is_empty() {
                    to_str_(s, &section.children);
                }
                s.push(']');
                if iterator.peek().is_some() {
                    s.push_str(", ");
                }
            }
        }

        let mut s = String::new();
        to_str_(&mut s, sections);
        s
    }

    #[track_caller]
    fn t(source: &str, expected: &str) {
        let mut o = fastn_section::Document::default();
        let sections = parse(source);
        let sections = super::inner_ender(source, &mut o, sections);
        assert_eq!(to_str(&sections), expected);
        // assert!(o.items.is_empty());
    }

    #[track_caller]
    fn f(source: &str, expected: &str, errors: Vec<fastn_section::Error>) {
        let mut o = fastn_section::Document::default();
        let sections = parse(source);
        let sections = super::inner_ender(source, &mut o, sections);
        assert_eq!(to_str(&sections), expected);

        assert_eq!(
            o.errors,
            errors
                .into_iter()
                .map(|value| fastn_section::Spanned {
                    span: fastn_section::utils::dummy_span(),
                    value,
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
            vec![fastn_section::Error::EndWithoutStart],
        );
        t("foo -> /foo", "foo");
        t("foo -> /foo -> bar", "foo, bar");
        t("bar -> foo -> /foo -> baz", "bar, foo, baz");
        t("bar -> a -> /a -> foo -> /foo -> baz", "bar, a, foo, baz");
        t(
            "bar -> a -> b -> /a -> foo -> /foo -> baz",
            "bar, a [b], foo, baz",
        );
        t("foo -> bar -> baz -> /bar -> /foo", "foo [bar [baz]]");
        t(
            "foo -> bar -> baz -> a -> /bar -> /foo",
            "foo [bar [baz, a]]",
        );
        t(
            "foo -> bar -> baz -> a -> /a -> /bar -> /foo",
            "foo [bar [baz, a]]",
        );
        t("bar -> bar -> baz -> /bar -> /bar", "bar [bar [baz]]");
        t("bar -> bar -> /bar -> /bar", "bar [bar]");
    }
}
