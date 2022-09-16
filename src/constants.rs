// Identifier constants
pub mod identifier {
    pub const SECTION: &str = "-- ";
    pub const SUBSECTION: &str = "--- ";
    pub const COMMENTED_SECTION: &str = "/-- ";
    pub const COMMENTED_SUBSECTION: &str = "/--- ";
    pub const ESCAPED_SECTION: &str = r"\-- ";
    pub const ESCAPED_SUBSECTION: &str = r"\--- ";
    pub const KV_SEPERATOR: &str = ":";
    pub const WHITESPACE: &str = " ";

    pub fn is_section(line: &str) -> bool {
        line.starts_with(SECTION)
    }
    pub fn is_section_commented(line: &str) -> bool {
        line.starts_with(COMMENTED_SECTION)
    }
    pub fn is_section_escaped(line: &str) -> bool {
        line.starts_with(ESCAPED_SECTION)
    }
    pub fn is_section_commented_or_escaped(line: &str) -> bool {
        is_section_commented(line) || is_section_escaped(line)
    }

    pub fn is_subsection(line: &str) -> bool {
        line.starts_with(SUBSECTION)
    }
    pub fn is_subsection_commented(line: &str) -> bool {
        line.starts_with(COMMENTED_SUBSECTION)
    }
    pub fn is_subsection_escaped(line: &str) -> bool {
        line.starts_with(ESCAPED_SUBSECTION)
    }
    pub fn is_subsection_commented_or_escaped(line: &str) -> bool {
        is_subsection_commented(line) || is_subsection_escaped(line)
    }

    pub fn is_section_or_subsection(line: &str) -> bool {
        is_section(line) || is_subsection(line)
    }
    pub fn is_commented_section_or_subsection(line: &str) -> bool {
        is_section_commented(line) || is_subsection_commented(line)
    }
    pub fn is_escaped_section_or_subsection(line: &str) -> bool {
        is_section_escaped(line) || is_subsection_escaped(line)
    }

    /// will trim any normal, commented or
    /// escaped section/subsection identifier from the beginning
    pub fn trim_section_subsection_identifier(line: &str) -> &str {
        line.trim_start_matches(|c| c == '/' || c == '\\' || c == '-' || c == ' ')
    }

    /// returns key/value pair seperated by KV_SEPERATOR
    pub fn segregate_key_value(
        line: &str,
        doc_id: &str,
        line_number: usize,
    ) -> ftd::p1::Result<(String, Option<String>)> {
        // Trim any section/subsection identifier from the beginning of the line
        let line = trim_section_subsection_identifier(line);

        let (before_kv_delimiter, after_kv_delimiter) =
            line.split_once(KV_SEPERATOR)
                .ok_or_else(|| ftd::p1::Error::NotFound {
                    doc_id: doc_id.to_string(),
                    line_number,
                    key: format!("\':\' not found while segregating kv in {}", line),
                })?;

        match (before_kv_delimiter, after_kv_delimiter) {
            (before, after) if after.trim().is_empty() => Ok((before.trim().to_string(), None)),
            (before, after) => Ok((before.trim().to_string(), Some(after.trim().to_string())))
        }
    }
}

// Regex pattern constants
pub mod regex {

    // Back references and arbitrary lookahead/lookbehind assertions
    // are not provided by rust regex so avoid using them,
    // instead do post process the matches to discard all unnecessary matches
    // or reformulate the regex to avoid any arbitrary
    // lookahead/lookbehind assertions
    // Refer issue - https://github.com/rust-lang/regex/issues/127

    /// Linking syntax: `<prefix>[<id_or_text>](<type1><id>)?`
    pub const LINK_SYNTAX: &str = r"(?x) # Enabling comment mode {GROUP 0 = entire match}
    (?P<prefix>.?) # Character Prefix Group <prefix>
    \[(?P<id_or_text>[-\s\w]+)\] # Referred Id Capture Group <id_or_text>
    (\(((?P<type1>\s*id\s*:(?P<id>.+))|(?P<ahead>.+\#.+))\))? # <type1> group and <ahead> group for any possible link";

    /// Linking Syntax 1: `[<linked-text>]`(id: `<id>`)
    pub const LINK_SYNTAX_1: &str = r"(?x) # Enabling Comment Mode {GROUP 0 = entire match}
    (?P<prefix>.?) # Character Prefix Group <prefix> {GROUP 1}
    \[(?P<linked_text>[-\s\w]+)\] # Linked Text Capture Group <linked_text> {GROUP 2}
    \(\s*id\s*:(?P<actual_id>[-\s\w]+)\) # Referred Id Capture Group <actual_id> {GROUP 3}";

    /// Linking Syntax 2: `[<id>]`
    ///
    /// Linked text is same as `<id>` in this case
    pub const LINK_SYNTAX_2: &str = r"(?x) # Enabling comment mode {GROUP 0 = entire match}
    (?P<prefix>.?) # Character Prefix Group <prefix> {GROUP 1}
    \[(?P<actual_id>[-\s\w]+)\] # Referred Id Capture Group <actual_id> {GROUP 2}
    (?P<ahead>(\(.+\#.+\))?) # Bracket Group <ahead> if any {GROUP 3}";

    /// id: `<alphanumeric string>` (with -, _, whitespace allowed)
    pub const ID_HEADER: &str = r"(?m)^\s*id\s*:[-\s\w]*$";

    /// file extension: \[.\]<alphanumeric string>$
    /// to cover all file extensions with file names
    /// ending with .ftd, .md, .jpg ... etc
    pub const FILE_EXTENSION: &str = r"[.][a-z\d]+[/]?$";

    lazy_static::lazy_static! {
        pub static ref ID: regex::Regex = regex::Regex::new(ID_HEADER).unwrap();
        pub static ref S: regex::Regex = regex::Regex::new(LINK_SYNTAX).unwrap();
        pub static ref S1: regex::Regex = regex::Regex::new(LINK_SYNTAX_1).unwrap();
        pub static ref S2: regex::Regex = regex::Regex::new(LINK_SYNTAX_2).unwrap();
        pub static ref EXT: regex::Regex = regex::Regex::new(FILE_EXTENSION).unwrap();
    }

    /// fetches capture group by group index and returns it as &str
    pub fn capture_group_by_index<'a>(capture: &'a regex::Captures, group_index: usize) -> &'a str {
        return capture.get(group_index).map_or("", |c| c.as_str());
    }

    /// fetches the capture group by group name and returns it as &str
    pub fn capture_group_by_name<'a>(capture: &'a regex::Captures, group_name: &str) -> &'a str {
        return capture.name(group_name).map_or("", |c| c.as_str());
    }
}
