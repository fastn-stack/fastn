/// Parses the value portion of a header or caption.
///
/// Header values are the content that appears after the colon in a header
/// or after the colon in a section initialization (caption). They can contain
/// plain text and will eventually support embedded expressions.
///
/// # Grammar
/// ```text
/// header_value = (text | expression)*
/// text = <any content except '{' or newline>
/// expression = '{' ... '}' (not yet implemented)
/// ```
///
/// # Parsing Rules
/// - Parses content from the current position until end of line
/// - Stops at newline character (does not consume it)
/// - Will eventually support expressions within `{}` braces
/// - Currently treats '{' as a terminator (expression support pending)
/// - Trailing whitespace is preserved as part of the value
///
/// # Examples
/// ```text
/// Simple text value
/// Value with spaces    
/// UTF-8 text: café, 日本語
/// Future: text with {expression}
/// ```
///
/// # Returns
/// Returns `Some(HeaderValue)` containing the parsed text segments,
/// or `None` if no content is found before the end of line.
pub fn header_value(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::HeaderValue> {
    let mut ses = Vec::new();
    while let Some(text) = scanner.take_till_char_or_end_of_line('{') {
        ses.push(fastn_section::Tes::Text(text));
        if !scanner.take('{') {
            // we have reached the end of the scanner
            break;
        }
    }

    if ses.is_empty() {
        return None;
    }

    Some(fastn_section::HeaderValue(ses))
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::header_value);

    #[test]
    fn tes() {
        t!("hello", ["hello"]);
        t!("hèllo", ["hèllo"]);
        // t!("hello ${world}", [{ "text": "hello $" }, /* expression containing "world" */], "");
    }
}
