pub fn qualified_identifier(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::QualifiedIdentifier> {
    let module = match fastn_section::module_name(scanner) {
        Some(module) => match scanner.peek() {
            Some('#') => {
                scanner.pop();
                Some(module)
            }
            _ => {
                return Some(fastn_section::QualifiedIdentifier {
                    module: Some(module),
                    terms: vec![],
                })
            }
        },
        None => None,
    };

    let terms = {
        let mut terms = Vec::new();
        while let Some(identifier) = fastn_section::identifier(scanner) {
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

    Some(fastn_section::QualifiedIdentifier::new(module, terms))
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::qualified_identifier);

    #[test]
    fn qualified_identifier() {
        t!("foo", "foo");
        t!("foo.com#bar", { "module": "foo.com as foo", "terms": ["bar"]});
        t!("foo.com#bar.baz", { "module": "foo.com as foo", "terms": ["bar", "baz"]});
        t!(
            "foo.com/yo#bar.baz",
            {"module": { "package": "foo.com as foo", "name": "yo"}, "terms": ["bar", "baz"]},
            ""
        );
        t!(
            "foo.com/yo/man#bar.baz",
            {
                "module": {
                    "package": "foo.com as foo",
                    "name": "man",
                    "path": ["yo"]
                },
                "terms": ["bar", "baz"]
            },
            ""
        );
        assert_eq!(
            super::qualified_identifier(&mut fastn_section::Scanner::new(
                " string",
                Default::default(),
                fastn_section::Document::default()
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
