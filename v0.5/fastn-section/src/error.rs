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
    SectionColonMissing,  // Missing colon after section name: -- foo
    HeaderColonMissing,   // Missing colon after header name: bar
    SectionNameNotFoundForEnd,
    EndContainsData,
    EndWithoutStart,
    ImportCantHaveType,
    ImportMustBeImport,
    ImportMustHaveCaption,
    ImportPackageNotFound,
    BodyNotAllowed,
    /// Body content found without required double newline separator after headers
    BodyWithoutDoubleNewline,
    /// Unclosed brace in expression
    UnclosedBrace,
    /// Wrong number of dashes in section marker (e.g., - or ---)
    DashCountError,
    /// Missing name in section declaration
    MissingName,
    /// Unclosed parenthesis in function marker
    UnclosedParen,
    ExtraArgumentFound,
    ArgumentValueRequired,
    ComponentIsNotAFunction,
    SymbolNotFound,
    InvalidIdentifier,
    UnexpectedCaption,
    InvalidPackageFile,
    PackageFileNotFound,
    // package: <caption> is either missing or is "complex"
    PackageNameNotInCaption,
    UnexpectedSectionInPackageFile,
    // FASTN.ftd does not contain `package:` declaration
    PackageDeclarationMissing,
    PackageNotFound,
    // SectionNotFound(&'a str),
    // MoreThanOneCaption,
    // ParseError,
    // MoreThanOneHeader,
    // HeaderNotFound,
}
