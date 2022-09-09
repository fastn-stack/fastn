// Identifier constants
pub mod identifier {
    pub const SECTION: &'static str = "-- ";
    pub const SUBSECTION: &'static str = "--- ";
    pub const COMMENTED_SECTION: &'static str = "/-- ";
    pub const COMMENTED_SUBSECTION: &'static str = "/--- ";
    pub const ESCAPED_SECTION: &'static str = r"\-- ";
    pub const ESCAPED_SUBSECTION: &'static str = r"\--- ";

    pub fn is_section(line: &str) -> bool {
        line.starts_with(SECTION)
    }
    pub fn is_subsection(line: &str) -> bool {
        line.starts_with(SUBSECTION)
    }

    pub fn is_commented_section(line: &str) -> bool {
        line.starts_with(COMMENTED_SECTION)
    }
    pub fn is_commented_subsection(line: &str) -> bool {
        line.starts_with(COMMENTED_SUBSECTION)
    }
    pub fn is_commented_section_or_subsection(line: &str) -> bool {
        is_commented_section(line) || is_commented_subsection(line)
    }

    pub fn is_section_escaped(line: &str) -> bool {
        line.starts_with(ESCAPED_SECTION)
    }
    pub fn is_subsection_escaped(line: &str) -> bool {
        line.starts_with(ESCAPED_SUBSECTION)
    }
    pub fn is_section_subsection_escaped(line: &str) -> bool {
        is_section_escaped(line) || is_subsection_escaped(line)
    }
}

// Character constants
pub mod character {
    pub const EMPTY: &'static str = "";
    pub const WHITESPACE: char = ' ';
    pub const TWO_SPACE: &'static str = "  ";
    pub const TAB_SPACE: &'static str = "   ";
    pub const COLON: char = ':';
    pub const SEMICOLON: char = ';';
}

// Regex pattern constants
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
