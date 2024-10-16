#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct ParseOutput<'a> {
    pub doc_name: &'a str,
    pub module_doc: Option<fastn_p1::Sourced<&'a str>>,
    pub items: Vec<fastn_p1::Sourced<fastn_p1::Item<'a>>>,
    /// length of each line in the source
    pub line_lengths: Vec<u8>,
}

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum Item<'a> {
    Section(fastn_p1::Section<'a>),
    Error(fastn_p1::Sourced<fastn_p1::SingleError<'a>>),
    Comment(&'a str),
}

// -- foo :
//       ^
//       |
//
// -- foo :
//        ^
//        |

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum SingleError<'a> {
    // foo
    SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}

impl fastn_p1::ParseOutput<'_> {
    /// read the module doc, and update the self.module_doc.
    ///
    /// this function returns the index of beginning of first line after the module doc.
    /// it enqueues all the comments before the module doc into the self.items.
    ///
    /// it also includes all the errors found, e.g., if it found any line that does not
    /// start with a section, nor is a comment.
    fn read_module_doc(&mut self, _e: &fastn_p1::Edit) -> usize {
        0
    }

    /// parse a single section
    ///
    /// this function parses till it finds a valid complete section, or encounters the end of
    /// the file.
    ///
    /// if it found the end of the file, it returns None, else it stops after the first character
    /// out of this section.
    ///
    /// it updates the properties of the section passed to this function.
    fn parse_section(
        &self,
        _section: &mut fastn_p1::Section,
        _index: usize,
        _e: &fastn_p1::Edit,
    ) -> Option<usize> {
        None
    }

    /// update() is an incremental parser
    ///
    /// parse_edit takes the result of last parse, and the latest edit operation, and updates the
    /// parse result.
    ///
    /// ```
    ///  let s = "-- foo:\n--bar:\n".to_string();
    ///  let mut engine = fastn_p1::ParserEngine::new("foo".to_string());
    ///  let mut output = fastn_p1::ParseOutput::default();
    ///  let edit = engine.add_edit(0, s.len(), s);
    ///  output.update(edit);
    /// ```
    pub fn update(&mut self, e: &fastn_p1::Edit) {
        // let's pretend we are not doing incremental parsing for now

        let mut index: usize = self.read_module_doc(e);

        let mut section = fastn_p1::Section::default();
        while let Some(end) = self.parse_section(&mut section, index, e) {
            self.items.push(fastn_p1::Sourced {
                from: index,
                to: end,
                value: fastn_p1::Item::Section(section),
            });
            index = end;
            section = fastn_p1::Section::default();
        }
    }
}
