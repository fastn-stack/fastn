/// identifier is variable or component etc name
///
/// identifier starts with Unicode alphabet and can contain any alphanumeric Unicode character
/// dash (`-`) and underscore (`_`) are also allowed
///
/// TODO: identifiers can't be keywords of the language, e.g., `import`, `record`, `component`.
/// but it can be built in types e.g., `integer` etc.
fn identifier(scanner: &mut fastn_p1::parser_v4::Scanner) -> Option<fastn_p1::Identifier> {
    let first = scanner.peek()?;
    // the first character should be is_alphabetic or `_`
    if !first.is_alphabetic() && first != '_' {
        return None;
    }

    let start = scanner.index();
    scanner.pop();

    // later characters should be is_alphanumeric or `_` or `-`
    while let Some(c) = scanner.peek() {
        if !c.is_alphanumeric() && c != '_' && c != '-' {
            break;
        }
        scanner.pop();
    }

    Some(fastn_p1::Identifier {
        name: fastn_p1::Span {
            start,
            end: scanner.index(),
        },
    })
}

#[cfg(test)]
mod test {
    macro_rules! t {
        ($source:expr, $debug:tt, $remaining:expr) => {
            fastn_p1::parser_v4::p(
                $source,
                super::identifier,
                serde_json::json!($debug),
                $remaining,
            );
        };
    }

    #[test]
    fn identifier() {
        // identifiers can't start with a space
        t!(" foo", null, " foo");
        t!("foo", "foo", "");
        t!("foo bar", "foo", " bar");
        t!("_foo bar", "_foo", " bar");
        t!("_foo-bar", "_foo-bar", "");
        t!("नम", "नम", "");
        t!("_नम-जन ", "_नम-जन", " ");
        t!("_नाम-जाने", "_नाम-जाने", "");
        t!("_नाम-जाने ", "_नाम-जाने", " ");
        // emoji is not a valid identifier
        t!("नम😦", "नम", "😦");
        t!("नम 😦", "नम", " 😦");
        t!("😦नम ", null, "😦नम ");
    }
}
