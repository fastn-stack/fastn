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
/// expression = '{' ... '}'
/// ```
///
/// # Returns
/// Returns `Some(HeaderValue)` containing the body text, or `None` if no body content exists.
pub fn body(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::HeaderValue> {
    // Create a terminator that stops at section markers or doc comments
    let body_terminator = |s: &mut fastn_section::Scanner<fastn_section::Document>| {
        // Save position to check ahead
        let check_index = s.index();
        s.skip_spaces();

        // Check for section markers
        if s.one_of(&["-- ", "/--"]).is_some() {
            s.reset(&check_index);
            return true;
        }

        // Check for doc comments (;;;)
        if s.peek() == Some(';') {
            let save = s.index();
            s.pop();
            if s.peek() == Some(';') {
                s.pop();
                if s.peek() == Some(';') {
                    // Found doc comment
                    s.reset(&check_index);
                    return true;
                }
            }
            s.reset(&save);
        }

        s.reset(&check_index);
        false
    };

    let tes = fastn_section::parser::tes_till(scanner, &body_terminator);

    if tes.is_empty() {
        return None;
    }

    Some(fastn_section::HeaderValue(tes))
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

        // Body stops at doc comment
        t!(
            "
            Some body text
            
            ;;; Doc comment
            -- next-section:",
            ["Some body text\n\n"],
            ";;; Doc comment\n-- next-section:"
        );

        // Body stops at doc comment with spaces
        t!(
            "
            Body content
                ;;; Indented doc comment
            -- section:",
            ["Body content\n"],
            "    ;;; Indented doc comment\n-- section:"
        );

        // Body with expression
        t!(
            "Hello {world}!",
            ["Hello ", {"expression": ["world"]}, "!"]
        );

        // Body with multiple expressions
        t!(
            "Start {expr1} middle {expr2} end",
            ["Start ", {"expression": ["expr1"]}, " middle ", {"expression": ["expr2"]}, " end"]
        );

        // Body with nested expressions
        t!(
            "Outer {inner {nested} text} more",
            ["Outer ", {"expression": ["inner ", {"expression": ["nested"]}, " text"]}, " more"]
        );

        // Body with expression preserving leading whitespace
        t_raw!(
            "    Indented {expression} text",
            ["    Indented ", {"expression": ["expression"]}, " text"]
        );

        // Body with expression across lines preserving indentation
        t_raw!(
            "Line one {expression\n    with indented\n        continuation} here",
            ["Line one ", {"expression": ["expression\n    with indented\n        continuation"]}, " here"]
        );

        // Body with deeply indented expression
        t_raw!(
            "        Deep indent {expr} text\n    More content",
            ["        Deep indent ", {"expression": ["expr"]}, " text\n    More content"]
        );

        // Body with tabs and spaces before expression
        t_raw!(
            "\t\tTabbed {content} here",
            ["\t\tTabbed ", {"expression": ["content"]}, " here"]
        );

        // Body with unclosed brace - now recovers
        t_err!(
            "Text before {unclosed", 
            ["Text before ", {"expression": ["unclosed"]}],
            "unclosed_brace"
        );

        // Body with expression followed by section
        t!(
            "
            Text {expr} more
            -- next:",
            ["Text ", {"expression": ["expr"]}, " more\n"],
            "-- next:"
        );
    }
}
