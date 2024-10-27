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

    #[test]
    fn test_identifier() {
        p("foo", super::identifier, serde_json::json!("foo"), "");
        p(
            "foo bar",
            super::identifier,
            serde_json::json!("foo"),
            " bar",
        );
        p(
            "_foo bar",
            super::identifier,
            serde_json::json!("_foo"),
            " bar",
        );
        p(
            "_foo-bar",
            super::identifier,
            serde_json::json!("_foo-bar"),
            "",
        );
        p("नम", super::identifier, serde_json::json!("नम"), "");
        p(
            "_नम-जन ",
            super::identifier,
            serde_json::json!("_नम-जन"),
            " ",
        );
        p("_नाम ", super::identifier, serde_json::json!("_नाम"), " ");
        p(
            "_नाम-जाने ",
            super::identifier,
            serde_json::json!("_नाम-जाने"),
            " ",
        );
    }
}
