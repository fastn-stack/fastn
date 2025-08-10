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
    let tes = fastn_section::parser::tes_till_newline(scanner)?;

    if tes.is_empty() {
        return None;
    }

    Some(fastn_section::HeaderValue(tes))
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::header_value);

    #[test]
    fn tes() {
        // Plain text
        t!("hello", "hello");
        t!("hèllo", "hèllo");

        // Text with expression
        t!("hello {world}", ["hello ", {"expression": ["world"]}]);

        // Multiple expressions
        t!("a {b} c", ["a ", {"expression": ["b"]}, " c"]);

        // Nested expressions
        t!("outer {inner {nested}}", [
            "outer ",
            {"expression": ["inner ", {"expression": ["nested"]}]}
        ]);

        // Empty expression
        t!("{}", [{"expression": []}]);

        // Unclosed brace - now recovers by consuming content
        t_err!(
            "hello {world",
            ["hello ", {"expression": ["world"]}],
            "unclosed_brace"
        );

        // Dollar expression - now properly handled
        t!("hello ${world}", ["hello ", {"$expression": ["world"]}]);

        // Mixed expressions
        t!("${a} and {b}", [{"$expression": ["a"]}, " and ", {"expression": ["b"]}]);

        // Dollar without brace
        t!("price: $100", "price: $100");
    }
}
