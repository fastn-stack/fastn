/// identifier is variable or component etc name
///
/// identifier starts with Unicode alphabet and can contain any alphanumeric Unicode character
/// dash (`-`) and underscore (`_`) are also allowed
///
/// identifiers can't be keywords of the language, e.g., `import`, `record`, `component`.
/// but it can be built in types e.g., `integer` etc.
fn identifier(scanner: &mut fastn_p1::parser_v4::scanner::Scanner) -> Option<fastn_p1::Identifier> {
    scanner.identifier().map(Into::into)
}

#[cfg(test)]
mod test {
    #[test]
    fn test_identifier() {
        use fastn_p1::debug::JDebug;

        let source = "foo";

        let mut scanner = fastn_p1::parser_v4::scanner::Scanner::new(source);
        let result = super::identifier(&mut scanner);
        assert_eq!(
            result.map(|v| v.debug(source)),
            Some(serde_json::json!("foo"))
        );
    }
}
