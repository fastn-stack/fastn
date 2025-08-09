/// Parses a kinded reference from the scanner.
///
/// A kinded reference consists of an optional type/kind followed by a name that
/// can be an identifier reference (allowing '.', '#', '/' for qualified names).
/// This is commonly used in section initialization where you can have:
/// - `-- foo:` (simple name)
/// - `-- string message:` (kind with simple name)
/// - `-- ftd.text:` (qualified name)
/// - `-- list<int> items:` (generic kind with name)
///
/// # Grammar
/// ```text
/// kinded_reference = [kind] spaces identifier_reference
/// ```
///
/// Only spaces and tabs are allowed between the kind and name (not newlines).
/// However, newlines ARE allowed within generic type parameters themselves.
///
/// # Special Cases
/// When parsing something ambiguous like `ftd.text`, it could be either:
/// - A qualified identifier reference (the name)
/// - A kind with module qualification
///
/// The parser handles this by first trying to parse as a kind, and if no
/// following identifier reference is found, it backtracks and parses the
/// whole thing as an identifier reference.
pub fn kinded_reference(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::KindedReference> {
    // First try to parse an optional kind (type)
    let start = scanner.index();
    let kind = fastn_section::parser::kind(scanner);

    // Check if we have a name after the kind
    scanner.skip_spaces();
    let name = fastn_section::parser::identifier_reference(scanner);

    // Determine what we actually parsed
    match (kind, name) {
        (Some(k), Some(n)) => {
            // We have both kind and name: "string foo" or "list<int> items"
            Some(fastn_section::KindedReference {
                kind: Some(k),
                name: n,
            })
        }
        (Some(k), None) => {
            // We have a kind but no following name.
            // Only treat it as a name if it's a simple kind (no generics)
            if k.args.is_some() {
                // Complex kind with generics, can't be just a name
                scanner.reset(&start);
                return None;
            }

            // It's a simple kind, could be a name
            // Reset and parse it as an identifier_reference
            scanner.reset(&start);
            let name = fastn_section::parser::identifier_reference(scanner)?;
            Some(fastn_section::KindedReference { kind: None, name })
        }
        (None, _) => {
            // Nothing parsed
            None
        }
    }
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::kinded_reference);

    #[test]
    fn kinded_reference() {
        // Simple cases
        t!("foo", {"name": "foo"});
        t!("string foo", {"name": "foo", "kind": "string"});

        // Qualified names
        t!("ftd.text", {"name": "ftd.text"});
        t!("module.component", {"name": "module.component"});
        t!("package#item", {"name": "package#item"});

        // Generic types with names
        t!("list<string> items", {"name": "items", "kind": {"name": "list", "args": ["string"]}});
        t!("map<string, int> data", {
            "name": "data",
            "kind": {"name": "map", "args": ["string", "int"]}
        });

        // Nested generics
        t!("map<string, list<int>> nested", {
            "name": "nested",
            "kind": {"name": "map", "args": ["string", {"name": "list", "args": ["int"]}]}
        });

        // Multiple spaces between kind and name
        t!("string    foo", {"name": "foo", "kind": "string"});

        // Underscores and numbers
        t!("string _private", {"name": "_private", "kind": "string"});
        t!("int value123", {"name": "value123", "kind": "int"});

        // Qualified names as the name part
        t!("string module.field", {"name": "module.field", "kind": "string"});
        t!("list<int> package#items", {"name": "package#items", "kind": {"name": "list", "args": ["int"]}});

        // Edge case: generic type without a following name fails
        // because "list<int>" can't be parsed as an identifier_reference
        f!("list<int>");
    }
}
