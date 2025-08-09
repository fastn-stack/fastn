/// Parses a plain identifier from the scanner.
///
/// A plain identifier can only contain:
/// - First character: alphabetic or underscore
/// - Subsequent characters: alphanumeric, underscore, or hyphen
///
/// This is used for simple, unqualified names like variable names, function names, etc.
/// For qualified names with dots, hashes, or slashes, use `identifier_reference` instead.
///
/// Examples:
/// - `foo`, `bar`, `test123`, `_private`, `my-var`, `नाम123`
pub fn identifier(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::Identifier> {
    let first = scanner.peek()?;
    // the first character should be is_alphabetic or `_`
    if !first.is_alphabetic() && first != '_' {
        return None;
    }

    // later characters should be is_alphanumeric or `_` or `-`
    let span = scanner.take_while(|c| c.is_alphanumeric() || c == '_' || c == '-')?;

    Some(fastn_section::Identifier { name: span })
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::identifier);

    #[test]
    fn identifier() {
        // identifiers can't start with a space
        t!(" foo", null, " foo");
        t!("foo", "foo");
        t!("foo bar", "foo", " bar");
        t!("_foo bar", "_foo", " bar");
        t!("_foo-bar", "_foo-bar");
        t!("नम", "नम");
        t!("_नम-जन ", "_नम-जन", " ");
        t!("_नाम-जाने", "_नाम-जाने");
        t!("_नाम-जाने ", "_नाम-जाने", " ");
        // emoji is not a valid identifier
        t!("नम😦", "नम", "😦");
        t!("नम 😦", "नम", " 😦");
        t!("😦नम ", null, "😦नम ");

        // identifiers with numbers (new feature)
        t!("foo123", "foo123");
        t!("test_42", "test_42");
        t!("var-2-name", "var-2-name");
        t!("_9lives", "_9lives");
        // can't start with a number
        t!("123foo", null, "123foo");
        t!("42", null, "42");
        // mixed alphanumeric with unicode
        t!("नाम123", "नाम123");
        t!("test123 bar", "test123", " bar");
    }
}
