/// The Wiggin Module - named after "Ender Wiggin" from Orson Scott Card's "Ender's Game"

/// Processes a list of sections and their nested children recursively.
///
/// This function performs a two-phase processing:
/// 1. **Recursive phase**: Processes all embedded sections within each section
/// 2. **Structure phase**: Organizes sections based on their start/end markers
///
/// # Algorithm
/// The function recursively processes nested sections first (depth-first),
/// then uses `inner_ender` to match `-- end: <name>` markers with their
/// corresponding section starts to build the proper hierarchy.
///
/// # Parameters
/// - `o`: The document to collect any errors during processing
/// - `sections`: Flat list of sections that may contain end markers
///
/// # Returns
/// A properly nested vector of sections where children are contained
/// within their parent sections based on end markers.
///
/// # Example
/// ```text
/// Input:  [-- foo:, -- bar:, -- end: bar, -- end: foo]
/// Output: [foo [bar []]]
/// ```
#[allow(dead_code)]
pub fn ender(
    o: &mut fastn_section::Document,
    sections: Vec<fastn_section::Section>,
) -> Vec<fastn_section::Section> {
    // recursive part
    let sections = sections.into_iter().map(|s| section_ender(o, s)).collect();

    // non recursive part
    inner_ender(o, sections)
}

/// Recursively processes a single section and all its components.
///
/// Applies the ender logic to:
/// - Caption (if present)
/// - All headers
/// - Body content (if present)  
/// - All child sections
///
/// This ensures that any embedded sections within captions, headers, or body
/// are properly structured before the section itself is processed.
fn section_ender(
    o: &mut fastn_section::Document,
    mut section: fastn_section::Section,
) -> fastn_section::Section {
    if let Some(caption) = section.caption {
        section.caption = Some(header_value_ender(o, caption));
    }
    section.headers = section
        .headers
        .into_iter()
        .map(|mut h| {
            h.value = header_value_ender(o, h.value);
            h
        })
        .collect();
    if let Some(body) = section.body {
        section.body = Some(header_value_ender(o, body));
    }
    section.children = ender(o, section.children);
    section
}

/// Processes embedded content within header values.
///
/// Header values can contain:
/// - Plain text
/// - Expressions (with nested header values)
/// - Embedded sections
///
/// This function recursively processes any nested structures within
/// the header value to ensure proper hierarchy.
fn header_value_ender(
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
                    content: header_value_ender(o, content),
                },
                fastn_section::Tes::Section(sections) => {
                    fastn_section::Tes::Section(ender(o, sections))
                }
            })
            .collect(),
    )
}

/// Converts a flat section list with `-- end: <section-name>` markers into a properly nested hierarchy.
///
/// This is the core algorithm that matches section end markers with their corresponding
/// start sections to build a tree structure. It uses a stack-based approach to handle
/// arbitrary nesting depth.
///
/// # Algorithm
/// 1. Iterate through sections sequentially
/// 2. Push regular sections onto a stack
/// 3. When an end marker is found, pop sections from the stack until finding the matching start
/// 4. Sections popped become children of the matched parent section
/// 5. Report errors for unmatched end markers
///
/// # Example
/// ```text
/// Input:  [{section: "foo"}, {section: "bar"}, "-- end: foo"]
/// Output: [{section: "foo", children: [{section: "bar"}]}]
/// ```
///
/// # Error Handling
/// If an end marker is found without a corresponding start section,
/// an `EndWithoutStart` error is added to the document's error list.
fn inner_ender<T: SectionProxy>(o: &mut fastn_section::Document, sections: Vec<T>) -> Vec<T> {
    let mut stack = Vec::new();
    'outer: for section in sections {
        match section.mark().unwrap() {
            // If the section is a start marker, push it onto the stack
            Mark::Start(_name) => {
                stack.push(section);
            }
            // If the section is an end marker, find the corresponding start marker in the stack
            Mark::End(e_name) => {
                let mut children = Vec::new(); // Collect children for the matching section
                while let Some(mut candidate) = stack.pop() {
                    match candidate.mark().unwrap() {
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

/// Represents whether a section starts or ends a hierarchical block.
///
/// Used by the ender algorithm to distinguish between:
/// - Regular sections that start a new scope (`Start`)
/// - End markers that close a scope (`End`)
enum Mark {
    /// A regular section that may contain children
    Start(String),
    /// An end marker (e.g., `-- end: foo`) that closes a section
    End(String),
}

/// Abstraction trait for section-like types to enable testing and modularity.
///
/// This trait allows the ender algorithm to work with both real `Section` types
/// and test doubles. It defines the minimal interface needed for the hierarchical
/// processing logic.
///
/// # Why a trait?
/// Using a trait here enables:
/// - Unit testing with simplified mock sections
/// - Potential reuse with different section representations
/// - Clear separation of the algorithm from the data structure
trait SectionProxy: Sized + std::fmt::Debug {
    /// returns the name of the section, and if it starts or ends the section
    fn mark(&self) -> Result<Mark, fastn_section::Error>;

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
    fn mark(&self) -> Result<Mark, fastn_section::Error> {
        if self.simple_name() != Some("end") {
            return Ok(Mark::Start(self.init.name.to_string()));
        }

        let caption = match self.caption.as_ref() {
            Some(caption) => caption,
            None => return Err(fastn_section::Error::SectionNameNotFoundForEnd),
        };

        if caption.0.len() > 1 {
            return Err(fastn_section::Error::EndContainsData);
        }

        let v = match caption.0.get(0) {
            Some(fastn_section::Tes::Text(span)) => {
                let v = span.str().trim();
                // if v is not a single word, we have a problem
                if v.contains(' ') || v.contains('\t') {
                    // SES::String cannot contain new lines.
                    return Err(fastn_section::Error::EndContainsData);
                }
                v
            }
            Some(_) => return Err(fastn_section::Error::EndContainsData),
            None => return Err(fastn_section::Error::SectionNameNotFoundForEnd),
        };

        Ok(Mark::End(v.to_string()))
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
        module: fastn_section::Module,
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
        fn mark(&self) -> Result<super::Mark, fastn_section::Error> {
            if self.has_end_mark {
                Ok(super::Mark::End(self.name.clone()))
            } else {
                Ok(super::Mark::Start(self.name.clone()))
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
            fastn_section::Span::with_module(self.module)
        }
    }

    // format: foo -> bar -> /foo (
    fn parse(name: &str, module: fastn_section::Module) -> Vec<DummySection> {
        let mut sections = vec![];
        let current = &mut sections;
        for part in name.split(" -> ") {
            let is_end = part.starts_with('/');
            let name = if is_end { &part[1..] } else { part };
            let section = DummySection {
                module,
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
        let mut arena = fastn_section::Arena::default();
        let module = fastn_section::Module::main(&mut arena);
        let mut o = fastn_section::Document {
            module,
            module_doc: None,
            sections: vec![],
            errors: vec![],
            warnings: vec![],
            comments: vec![],
            line_starts: vec![],
        };
        let sections = parse(source, module);
        let sections = super::inner_ender(&mut o, sections);
        assert_eq!(to_str(&sections), expected);
        // assert!(o.items.is_empty());
    }

    #[track_caller]
    fn f(source: &str, expected: &str, errors: Vec<fastn_section::Error>) {
        let mut arena = fastn_section::Arena::default();
        let module = fastn_section::Module::main(&mut arena);
        let mut o = fastn_section::Document {
            module,
            module_doc: None,
            sections: vec![],
            errors: vec![],
            warnings: vec![],
            comments: vec![],
            line_starts: vec![],
        };
        let sections = parse(source, module);
        let sections = super::inner_ender(&mut o, sections);
        assert_eq!(to_str(&sections), expected);

        assert_eq!(
            o.errors,
            errors
                .into_iter()
                .map(|value| fastn_section::Spanned {
                    span: fastn_section::Span::with_module(module),
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
