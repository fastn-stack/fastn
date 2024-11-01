pub fn qualified_identifier(
    scanner: &mut fastn_parser::section::Scanner,
) -> Option<fastn_parser::QualifiedIdentifier> {
    let module = match fastn_parser::section::module_name(scanner) {
        Some(module) => match scanner.peek() {
            Some('#') => {
                scanner.pop();
                Some(module)
            }
            _ => {
                return Some(fastn_parser::QualifiedIdentifier {
                    module: Some(module),
                    terms: vec![],
                })
            }
        },
        None => None,
    };

    let terms = {
        let mut terms = Vec::new();
        while let Some(identifier) = fastn_parser::section::identifier(scanner) {
            terms.push(identifier);
            if !scanner.take('.') {
                break;
            }
        }
        terms
    };

    if module.is_none() && terms.is_empty() {
        return None;
    }

    Some(fastn_parser::QualifiedIdentifier::new(module, terms))
}

#[cfg(test)]
mod test {
    fastn_parser::tt!(super::qualified_identifier);

    #[test]
    fn qualified_identifier() {
        t!("foo", "foo");
        t!("foo.com#bar", { "module": "foo.com", "terms": ["bar"]});
        t!("foo.com#bar.baz", { "module": "foo.com", "terms": ["bar", "baz"]});
        t!(
            "foo.com/yo#bar.baz",
            {"module": { "package": "foo.com", "path": ["yo"]}, "terms": ["bar", "baz"]},
            ""
        );
        t!(
            "foo.com/yo/man#bar.baz",
            {"module": { "package": "foo.com", "path": ["yo", "man"]}, "terms": ["bar", "baz"]},
            ""
        );
        assert_eq!(
            super::qualified_identifier(&mut fastn_parser::section::Scanner::new(
                " string",
                Default::default()
            ),),
            None
        );
        f!(" foo");
        f!(" string");
        f!(" foo.com#bar");
        f!(" foo.com/foo#bar");
        f!(" foo.com/foo#bar.bar");
    }
}
