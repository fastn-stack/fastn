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
    macro_rules! i {
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
        i!(" foo", null, " foo");
        i!("foo", "foo", "");
        i!("foo bar", "foo", " bar");
        i!("_foo bar", "_foo", " bar");
        i!("_foo-bar", "_foo-bar", "");
        i!("नम", "नम", "");
        i!("_नम-जन ", "_नम-जन", " ");
        i!("_नाम-जाने", "_नाम-जाने", "");
        i!("_नाम-जाने ", "_नाम-जाने", " ");
        // emoji is not a valid identifier
        i!("नम😦", "नम", "😦");
        i!("नम 😦", "नम", " 😦");
        i!("😦नम ", null, "😦नम ");
    }
}
