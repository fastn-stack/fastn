#![allow(dead_code)]

use std::fmt::Debug;

/// calls `inner_ender` for all the embedded section inside section in the
/// list and then calls `ender` for the list itself
pub fn ender(
    source: &str,
    o: &mut fastn_lang::unresolved::Document,
    sections: Vec<fastn_lang::Section>,
) -> Vec<fastn_lang::Section> {
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
    o: &mut fastn_lang::unresolved::Document,
    mut section: fastn_lang::Section,
) -> fastn_lang::Section {
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
    o: &mut fastn_lang::unresolved::Document,
    header: fastn_lang::HeaderValue,
) -> fastn_lang::HeaderValue {
    header
        .into_iter()
        .map(|ses| match ses {
            fastn_lang::SES::String(span) => fastn_lang::SES::String(span),
            fastn_lang::SES::Expression {
                start,
                end,
                content,
            } => fastn_lang::SES::Expression {
                start,
                end,
                content: header_value_ender(source, o, content),
            },
            fastn_lang::SES::Section(sections) => {
                fastn_lang::SES::Section(ender(source, o, sections))
            }
        })
        .collect()
}

/// converts a section list, with interleaved `-- end: <section-name>`, into a nested section list
///
/// example:
/// [{section: "foo"}, {section: "bar"}, "-- end: foo"] -> [{section: "foo", children: [{section: "bar"}]}]
fn inner_ender<T: SectionProxy>(
    source: &str,
    o: &mut fastn_lang::unresolved::Document,
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
                                children.insert(0, candidate);
                            }
                        }
                        Mark::End(_name) => {
                            // There is two possibilities here
                            // 1. name == e_name
                            // This could happen when the child section has same name as the parent
                            // -- foo:
                            //    -- foo:
                            //    -- end: foo
                            // -- end: foo
                            //
                            // 2. name != e_name
                            // This could happen when the child section has different name
                            // -- foo:
                            //    -- bar:
                            //    -- end: bar
                            // -- end: foo
                            // In both cases we want to add the child section to the list
                            children.insert(0, candidate);
                        }
                    }
                }
                // we have run out of sections, and we have not found the section end, return
                // error, put the children back on the stack
                o.errors.push(fastn_lang::Spanned {
                    span: section.span(),
                    value: fastn_lang::Error::EndWithoutStart,
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
trait SectionProxy: Sized + Debug {
    /// returns the name of the section, and if it starts or ends the section
    fn mark<'input>(&'input self, source: &'input str) -> Result<Mark<'input>, fastn_lang::Error>;
    fn add_children(&mut self, children: Vec<Self>);
    fn span(&self) -> fastn_lang::Span;
}

impl SectionProxy for fastn_lang::Section {
    fn mark<'input>(&'input self, source: &'input str) -> Result<Mark<'input>, fastn_lang::Error> {
        let span = &self.init.name.name.name;
        let name = &source[span.start..span.end];
        if name != "end" {
            return Ok(Mark::Start(name));
        }

        let caption = match self.caption.as_ref() {
            Some(caption) => caption,
            None => return Err(fastn_lang::Error::SectionNameNotFoundForEnd),
        };

        let v = match (caption.get(0), caption.len()) {
            (Some(fastn_lang::SES::String(span)), 1) => &source[span.start..span.end].trim(),
            (Some(_), _) => return Err(fastn_lang::Error::EndContainsData),
            (None, _) => return Err(fastn_lang::Error::SectionNameNotFoundForEnd),
        };

        // if v is not a single word, we have a problem
        if v.contains(' ') || v.contains('\t') {
            // SES::String cannot contain new lines.
            return Err(fastn_lang::Error::EndContainsData);
        }

        Ok(Mark::End(v))
    }

    fn add_children(&mut self, children: Vec<Self>) {
        self.children = children;
    }

    fn span(&self) -> fastn_lang::Span {
        self.init.dashdash.clone()
    }
}

#[cfg(test)]
mod test {
    #[allow(dead_code)] // #[expect(dead_code)] is not working
    #[derive(Debug)]
    struct DummySection {
        name: String,
        is_end: bool,
        children: Vec<DummySection>,
    }

    impl super::SectionProxy for DummySection {
        fn mark<'input>(
            &'input self,
            _source: &'input str,
        ) -> Result<super::Mark<'input>, fastn_lang::Error> {
            if self.is_end {
                Ok(super::Mark::End(&self.name))
            } else {
                Ok(super::Mark::Start(&self.name))
            }
        }

        fn add_children(&mut self, children: Vec<Self>) {
            self.children = children;
        }

        fn span(&self) -> fastn_lang::Span {
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
        let mut o = fastn_lang::unresolved::Document::default();
        let sections = parse(source);
        let sections = super::inner_ender(source, &mut o, sections);
        assert_eq!(to_str(&sections), expected);
        // assert!(o.items.is_empty());
    }

    #[track_caller]
    fn f(source: &str, expected: &str, errors: Vec<fastn_lang::Error>) {
        let mut o = fastn_lang::unresolved::Document::default();
        let sections = parse(source);
        let sections = super::inner_ender(source, &mut o, sections);
        assert_eq!(to_str(&sections), expected);

        assert_eq!(
            o.errors,
            errors
                .into_iter()
                .map(|value| fastn_lang::Spanned {
                    span: Default::default(),
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
            vec![fastn_lang::Error::EndWithoutStart],
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
    }
}
