// Identifier constants
pub mod identifier {
    pub const SECTION: &str = "-- ";
    pub const SUBSECTION: &str = "--- ";
    pub const COMMENTED_SECTION: &str = "/-- ";
    pub const COMMENTED_SUBSECTION: &str = "/--- ";
    pub const ESCAPED_SECTION: &str = r"\-- ";
    pub const ESCAPED_SUBSECTION: &str = r"\--- ";
    pub const KV_SEPERATOR: &str = ":";

    pub fn is_section(line: &str) -> bool {
        line.starts_with(SECTION)
    }
    pub fn is_subsection(line: &str) -> bool {
        line.starts_with(SUBSECTION)
    }
    pub fn is_section_or_subsection(line: &str) -> bool {
        is_section(line) || is_subsection(line)
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

    /// will trim any normal, commented or
    /// escaped section/subsection identifier from the beginning
    pub fn trim_section_subsection_identifier(line: &str) -> &str {
        line.trim_start_matches(|c| c == '/' || c == '\\' || c == '-' || c == ' ')
    }

    /// returns key/value pair seperated by ':'
    pub fn segregate_key_value(
        line: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::p1::Result<(String, Option<String>)> {
        if !line.contains(KV_SEPERATOR) {
            return Err(ftd::p1::Error::ParseError {
                message: format!(": is missing in: {}", line),
                doc_id: doc_id.to_string(),
                line_number,
            });
        }

        // Trim any section/subsection identifier fron the beginning of the line
        let line = trim_section_subsection_identifier(line);

        let mut parts = line.splitn(2, KV_SEPERATOR);
        match (parts.next(), parts.next()) {
            (Some(name), Some(value)) => {
                // some key and some non-empty value
                Ok((name.to_string(), Some(value.trim().to_string())))
            }
            (Some(name), None) => {
                // some key with no value
                Ok((name.to_string(), None))
            }
            _ => Err(ftd::p1::Error::ParseError {
                message: format!("Unknown KV line found \'{}\'", line),
                doc_id: doc_id.to_string(),
                line_number,
            }),
        }
    }
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
    pub const ID_HEADER: &str = r"(?m)^\s*id\s*:[-_\sA-Za-z\d]*$";
}
