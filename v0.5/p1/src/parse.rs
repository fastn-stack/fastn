#[derive(Debug, PartialEq, Clone, Default, serde::Serialize)]
pub struct ParseOutput<'a> {
    pub doc_name: &'a str,
    pub module_doc: Option<fastn_p1::Sourced<std::borrow::Cow<'a, &'a str>>>,
    pub items: Vec<fastn_p1::Sourced<fastn_p1::Item<'a>>>,
    /// index of the last new line character in the source. we need to count line lengths
    last_new_line_at: usize,
    /// length of each line in the source
    pub line_lengths: Vec<usize>,
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
    /// we found some text when we were not expecting, eg at the beginning of the file before
    /// any section started, or inside a section that does not expect any text. this second part
    /// I am not sure right now as we are planning ot convert all text to text nodes inside a
    /// section. so by the end maybe this will only contain the first part.
    UnwantedTextFound(fastn_p1::Sourced<&'a [char]>),
    // SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}

#[allow(dead_code)]
enum CommentConsumed {
    Yes,
    NotComment,
    DocComment,
}

impl fastn_p1::ParseOutput<'_> {
    fn register_new_line(&mut self, index: usize) {
        self.line_lengths.push(index - self.last_new_line_at);
        self.last_new_line_at = index;
    }

    /// consume unwanted text till the next line
    ///
    /// this function adds the unwanted text, till the end of line, to the `self.items` as an
    /// `Error` item.
    ///
    /// when looking for the end of file, it can also find a comment, in which case the comment
    /// should be added to `self.items`, and a non-comment text should be added to the error items.
    ///
    /// it does not always add a new error item if the last error item was of the same type, and
    /// the index reflects its right after the last error, it appends the text to the last error.
    ///
    /// in case the text is like this:
    ///
    /// ```ftd
    /// hello ;; some comment                -- error added by previous invocation of this function
    /// world ;; some other comment          -- this invocation index pointed to start of this line
    /// ```
    ///
    /// two comments are found, and they are added to the `self.items`, and the text `hello` and
    /// `world` should be added to the `self.items` as an error with value: `hello\nworld` we have
    /// to capture the new line character as well.
    ///
    /// since this function ends at a newline, the self.last_new_line_at and self.line_lengths
    /// should be updated by this method.
    ///
    /// the index must point to the first character after the newline character when this function
    /// returns. returns true if the end of file is found.
    fn consume_unwanted_text_till_new_line(
        &mut self,
        _index: &mut usize,
        _e: &fastn_p1::Edit,
    ) -> bool {
        todo!()
    }

    /// this functions adds the text from the current index till the end of the line to the
    /// `self.items` as a comment.
    ///
    /// the index points to the beginning of first `;` character, and it should point to the first
    /// character after the newline character when this function returns.
    ///
    /// if it successfully found a comment, eg second character was also `;` but third character
    /// was not, it should return `CommentConsumed::Yes`, if it found three `;;;` it should return
    /// `CommentConsumed::DocComment`, and if it found a non-comment text (eg there was no second
    /// `;`, it should return `CommentConsumed::NotComment`.
    fn consume_line_comment(&mut self, _index: &mut usize, _e: &fastn_p1::Edit) -> CommentConsumed {
        todo!()
    }
    /// read the module doc, and update the self.module_doc.
    ///
    /// this function returns the index of beginning of first line after the module doc.
    /// it enqueues all the comments before the module doc into the self.items.
    ///
    /// it also includes all the errors found, e.g., if it found any line that does not
    /// start with a section, nor is a comment.
    ///
    /// this function returns None if it found the end of the file.
    fn read_module_doc(&mut self, e: &fastn_p1::Edit) -> Option<usize> {
        // TODO(non-incremental): this function is supposed to be incremental, but we are not
        let mut index = 0;
        loop {
            if e.text.len() <= index {
                return None;
            }

            match e.text.get(index) {
                Some('-') => {
                    break;
                }
                Some(' ') => {
                    index += 1;
                }
                Some('\n') => {
                    self.register_new_line(index);
                    index += 1;
                }
                Some(';') => {
                    match self.consume_line_comment(&mut index, e) {
                        CommentConsumed::Yes => {}
                        CommentConsumed::NotComment => {
                            // Not a comment, and not a section, so it is an error, eat everything
                            // till the next new line
                            self.consume_unwanted_text_till_new_line(&mut index, e);
                        }
                        CommentConsumed::DocComment => {
                            // we have found the first line of doc comment. we have to extract the
                            // doc comment till the end of the current line, or till we encounter
                            // a comment, eg `;;; some doc comment ;; some comment`, here the `some
                            // doc comment` is doc comment, and `some comment` is a comment.
                            // the comment should be added to the self.items, as comment, and the
                            // next line has to be evaluated, if that is a doc comment too, it should
                            // be appended with the `some doc comment\n` from this line.
                            todo!()
                        }
                    }
                }
                Some(_) => {
                    if self.consume_unwanted_text_till_new_line(&mut index, e) {
                        return None;
                    }
                }
                None => return None,
            }
        }
        Some(index)
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
        // TODO(non-incremental): this function is supposed to be incremental, but we are not

        // we have to reset these if this is an edit instead of first parse
        // self.last_new_line_at = 0;
        // self.line_lengths = vec![];

        let mut index: usize = match self.read_module_doc(e) {
            Some(index) => index,
            None => return,
        };

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
