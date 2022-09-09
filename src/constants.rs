// Identifiers constants
pub mod identifier {
    pub const SECTION: &'static str = "-- ";
    pub const SUBSECTION: &'static str = "--- ";
    pub const COMMENTED_SECTION: &'static str = "/-- ";
    pub const COMMENTED_SUBSECTION: &'static str = "/--- ";
    pub const ESCAPED_SECTION: &'static str = r"\-- ";
    pub const ESCAPED_SUBSECTION: &'static str = r"\--- ";
}

// Character/ Space Constants
pub mod character {
    pub const WHITESPACE: char = ' ';
    pub const TWO_SPACE: &'static str = "  ";
    pub const TAB_SPACE: &'static str = "   ";
    pub const COLON: char = ':';
    pub const SEMICOLON: char = ';';
    pub const EMPTY: &'static str = "";
}

// Regex patterns
pub mod regex_consts {
    /// Linking Syntax 1: `[<linked-text>]`(id: <some-id>)
    pub const LINK_SYNTAX_1: &str = r"(?x) # Enabling Comment Mode
    \[(?P<linked_text>[\sa-zA-Z\d]+)\] # Linked Text Capture Group <linked_text>
    \(\s*id\s*:(?P<actual_id>[\sa-zA-Z\d]+)\) # Referred Id Capture Group <actual_id>";

    /// Linking Syntax 2: {<some-id>}
    pub const LINK_SYNTAX_2: &str = r"(?x) # Enabling comment mode
    \{\s* # Here Linked Text is same as Referred Id
    (?P<actual_id>[\sa-zA-Z\d]+)\} # Referred Id Capture Group <actual_id>";

    /// id: `<alphanumeric string>` (with -, _, whitespace allowed)
    pub const ID_HEADER: &'static str = r"(?m)^\s*id\s*:[-_\sA-Za-z\d]*$";
}
