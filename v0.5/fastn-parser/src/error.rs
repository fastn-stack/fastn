#[derive(Debug, PartialEq, Clone, serde::Serialize, serde::Deserialize)]
pub enum Error {
    /// doc comments should either come at the beginning of the file as a contiguous chunk
    /// or right before a section or a header.
    UnexpectedDocComment,
    /// we found some text when we were not expecting, e.g., at the beginning of the file before
    /// any section started, or inside a section that does not expect any text. this second part,
    /// I am not sure right now as we are planning to convert all text to text nodes inside a
    /// section. so by the end, maybe this will only contain the first part.
    UnwantedTextFound,
    /// we found something like `-- list<> foo:`, type is not specified
    EmptyAngleText,
    /// we are looking for dash-dash, but found something else
    DashDashNotFound,
    KindedNameNotFound,
    ColonNotFound,
    SectionNameNotFoundForEnd,
    EndContainsData,
    EndWithoutStart,
    // SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}
