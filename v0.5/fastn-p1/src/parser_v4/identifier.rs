/// identifier is variable or component etc name
///
/// identifier starts with Unicode alphabet and can contain any alphanumeric Unicode character
/// dash (`-`) and underscore (`_`) are also allowed
///
/// identifiers can't be keywords of the language, e.g., `import`, `record`, `component`.
/// but it can be built in types e.g., `integer` etc.
fn identifier(scanner: &mut fastn_p1::parser_v4::Scanner) -> Option<fastn_p1::Identifier> {
    scanner.identifier().map(Into::into)
}

#[cfg(test)]
mod test {
    #[track_caller]
    fn p<T: fastn_p1::debug::JDebug, F: FnOnce(&mut fastn_p1::parser_v4::Scanner) -> T>(
        source: &str,
        f: F,
        debug: serde_json::Value,
        remaining: &str,
    ) {
        let mut scanner = fastn_p1::parser_v4::Scanner::new(source);
        let result = f(&mut scanner);
        assert_eq!(result.debug(source), debug);
        assert_eq!(scanner.remaining(), remaining);
    }

    macro_rules! i {
        ($source:expr, $debug:tt, $remaining:expr) => {
            p(
                $source,
                super::identifier,
                serde_json::json!($debug),
                $remaining,
            );
        };
    }

    #[test]
    fn test_identifier() {
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
