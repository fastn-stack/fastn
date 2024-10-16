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

#[derive(Debug, PartialEq, Clone, serde::Serialize)]
pub enum SingleError<'a> {
    SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}

impl fastn_p1::ParseOutput<'_> {
    /// parse_edit is an incremental parser
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
    pub fn update(&mut self, _e: &fastn_p1::Edit) {}
}
