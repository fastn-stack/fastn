/// identifier is variable or component etc name
///
/// identifier starts with Unicode alphabet and can contain any alphanumeric Unicode character
/// dash (`-`) and underscore (`_`) are also allowed
///
/// identifiers can't be keywords of the language, e.g., `import`, `record`, `component`. but
/// can be built in types e.g., `integer` etc.
fn identifier(
    _scanner: &mut fastn_p1::parser_v4::scanner::Scanner,
) -> Option<fastn_p1::Identifier> {
    todo!()
}

#[cfg(test)]
mod test {
    fn test_identifier() {
        let mut scanner = fastn_p1::parser_v4::scanner::Scanner::new("foo");
        let result = super::identifier(&mut scanner);
        assert_eq!(
            result,
            Some(fastn_p1::Identifier {
                term: fastn_p1::Span { start: 0, end: 3 }
            })
        );
    }
}
