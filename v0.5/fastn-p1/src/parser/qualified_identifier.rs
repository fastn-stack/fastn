/// module name looks like <module-name>#<identifier>
pub fn qualified_identifier(
    scanner: &mut fastn_p1::parser::Scanner,
) -> Option<fastn_p1::QualifiedIdentifier> {
    let module = match fastn_p1::parser::module_name(scanner) {
        Some(module) => match scanner.peek() {
            Some('#') => {
                scanner.pop();
                Some(module)
            }
            _ => {
                return Some(fastn_p1::QualifiedIdentifier {
                    module: Some(module),
                    terms: vec![],
                })
            }
        },
        None => None,
    };

    let terms = {
        let mut terms = Vec::new();
        while let Some(identifier) = fastn_p1::parser::identifier(scanner) {
            terms.push(identifier);
            if !scanner.take('.') {
                break;
            }
        }
        terms
    };

    Some(fastn_p1::QualifiedIdentifier { module, terms })
}

#[cfg(test)]
mod test {
    macro_rules! t {
        ($source:expr, $debug:tt, $remaining:expr) => {
            fastn_p1::parser::p(
                $source,
                super::qualified_identifier,
                serde_json::json!($debug),
                $remaining,
            );
        };
    }

    #[test]
    fn qualified_identifier() {
        t!("foo", { "module": { "package": "foo"}}, "");
        t!("foo.com#bar", { "module": { "package": "foo.com"}, "terms": ["bar"]}, "");
        t!("foo.com#bar.baz", { "module": { "package": "foo.com"}, "terms": ["bar", "baz"]}, "");
    }
}
