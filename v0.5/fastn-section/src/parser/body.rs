/// Parses body content from the scanner.
///
/// Body content is free-form text that appears after headers in a section,
/// separated by a double newline. The body continues until:
/// - Another section is encountered (starting with `-- ` or `/--`)
/// - The end of the input is reached
///
/// # Grammar
/// ```text
/// body = (text | expression)*
/// text = <any content except '{' or section markers>
/// expression = '{' ... '}' (not yet implemented)
/// ```
///
/// # Returns
/// Returns `Some(HeaderValue)` containing the body text, or `None` if no body content exists.
pub fn body(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::HeaderValue> {
    let mut ses = Vec::new();
    let start = scanner.index();
    let mut reset_index = scanner.index();
    while scanner.one_of(&["-- ", "/--"]).is_none() {
        scanner.take_till_char_or_end_of_line('{');

        if scanner.peek() == Some('{') {
            todo!();  // Expression support to be implemented
        }

        if !scanner.take('\n') {
            // we have reached the end of the scanner
            ses.push(fastn_section::Tes::Text(scanner.span(start)));
            return Some(fastn_section::HeaderValue(ses));
        }

        reset_index = scanner.index();
    }

    scanner.reset(reset_index);
    ses.push(fastn_section::Tes::Text(scanner.span(start)));
    Some(fastn_section::HeaderValue(ses))
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::body);

    #[test]
    fn body() {
        // Simple single line body
        t!("hello world", ["hello world"]);
        
        // Multi-line body
        t!(
            "
            hello 
             world",
            ["hello \n world"]
        );
        
        // Body stops at section marker (single newline before marker)
        t!(
            "
            hello 
             world
            -- foo:",
            ["hello \n world\n"],
            "-- foo:"
        );
        
        // Body stops at section marker (double newline before marker)
        t!(
            "
            hello 
             world

            -- foo:",
            ["hello \n world\n\n"],
            "-- foo:"
        );
        
        // Body stops at commented section marker
        t!(
            "
            hello 
             world

            /-- foo:",
            ["hello \n world\n\n"],
            "/-- foo:"
        );
        
        // Body with multiple paragraphs
        t!(
            "
            First paragraph
            with multiple lines
            
            Second paragraph
            also with text",
            ["First paragraph\nwith multiple lines\n\nSecond paragraph\nalso with text"]
        );
        
        // Body with indented text
        t!(
            "
            Some text
                indented content
                more indented
            back to normal",
            ["Some text\n    indented content\n    more indented\nback to normal"]
        );
        
        // Empty lines in body
        t!(
            "
            Line 1
            
            
            Line 2",
            ["Line 1\n\n\nLine 2"]
        );
        
        // Body ending at EOF without newline
        t!("no newline at end", ["no newline at end"]);
    }
}
