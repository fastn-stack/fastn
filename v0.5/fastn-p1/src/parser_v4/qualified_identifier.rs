/// module name looks like <module-name>#<identifier>
pub fn qualified_identifier(
    scanner: &mut fastn_p1::parser_v4::Scanner,
) -> Option<fastn_p1::QualifiedIdentifier> {
    let module = match fastn_p1::parser_v4::module_name(scanner) {
        Some(m) => {
            if !scanner.take('#') {
                return None;
            }
            Some(m)
        }
        None => None,
    };

    let terms = {
        let mut terms = Vec::new();
        while let Some(identifier) = fastn_p1::parser_v4::identifier(scanner) {
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
            fastn_p1::parser_v4::p(
                $source,
                super::qualified_identifier,
                serde_json::json!($debug),
                $remaining,
            );
        };
    }

    #[test]
    fn qualified_identifier() {
        t!("foo", null, "");
        // t!("foo.com/", null, "");
        // t!("foo.com/ ", null, " ");
        // t!("foo.com/asd", {"package":"foo.com", "path": ["asd"]}, "");
    }
}
