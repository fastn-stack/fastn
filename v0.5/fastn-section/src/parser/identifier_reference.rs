pub fn identifier_reference(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::IdentifierReference> {
    let first = scanner.peek()?;
    // the first character should be is_alphabetic or `_`
    if !first.is_alphabetic() && first != '_' {
        return None;
    }

    // later characters should be is_alphanumeric or `_` or `-`
    let span = scanner
        .take_while(|c| c.is_alphabetic() || c == '_' || c == '-' || c == '.' || c == '#')?;

    match from_span(span.clone()) {
        Ok(v) => Some(v),
        Err(e) => {
            scanner.add_error(span, e);
            None
        }
    }
}

fn from_span(
    span: fastn_section::Span,
) -> Result<fastn_section::IdentifierReference, fastn_section::Error> {
    Ok(fastn_section::IdentifierReference::Local(span))
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::identifier_reference);

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
    }
}
