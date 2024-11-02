pub fn qualified_identifier(
    scanner: &mut fastn_lang::Scanner<fastn_lang::token::Document>,
) -> Option<fastn_lang::token::QualifiedIdentifier> {
    let module = match fastn_lang::token::module_name(scanner) {
        Some(module) => match scanner.peek() {
            Some('#') => {
                scanner.pop();
                Some(module)
            }
            _ => {
                return Some(fastn_lang::token::QualifiedIdentifier {
                    module: Some(module),
                    terms: vec![],
                })
            }
        },
        None => None,
    };

    let terms = {
        let mut terms = Vec::new();
        while let Some(identifier) = fastn_lang::token::identifier(scanner) {
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

    Some(fastn_lang::token::QualifiedIdentifier::new(module, terms))
}

#[cfg(test)]
mod test {
    fastn_lang::tt!(super::qualified_identifier);

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
            super::qualified_identifier(&mut fastn_lang::Scanner::new(
                " string",
                Default::default(),
                fastn_lang::token::Document::default()
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
