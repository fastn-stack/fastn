/// Parses a kinded name from the scanner.
///
/// A kinded name consists of an optional type/kind followed by a name.
/// This is commonly used in function parameters, variable declarations, and
/// component definitions where you specify both the type and the name.
///
/// # Grammar
/// ```text
/// kinded_name = [kind] spaces name
/// ```
///
/// # Examples
/// - `string foo` - type "string" with name "foo"
/// - `list<int> items` - generic type "list<int>" with name "items"
/// - `foo` - just a name "foo" with no explicit type
/// - `map<string, list<int>> data` - nested generic type with name "data"
///
/// Only spaces and tabs are allowed between the kind and name (not newlines).
/// However, newlines ARE allowed within generic type parameters themselves,
/// e.g., `list<\n  int\n> items` is valid.
///
/// # Special Cases
/// If only a simple kind is present (like `string` or `foo`), it's interpreted
/// as a name without a type through the `From<Kind>` implementation.
/// However, complex kinds with generics (like `list<int>`) or qualified names
/// (like `module.Type`) cannot be converted to plain identifiers and will
/// cause the parse to fail.
pub fn kinded_name(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::KindedName> {
    let start = scanner.index();
    let kind = fastn_section::parser::kind(scanner);
    scanner.skip_spaces(); // Only spaces/tabs between kind and name, not newlines

    let name = match fastn_section::parser::identifier(scanner) {
        Some(v) => v,
        None => {
            // If we have a kind but no name, try to convert the kind to a name
            match kind.and_then(Into::into) {
                Some(kinded_name) => return Some(kinded_name),
                None => {
                    // Conversion failed, backtrack
                    scanner.reset(start);
                    return None;
                }
            }
        }
    };

    Some(fastn_section::KindedName { kind, name })
}

impl From<fastn_section::Kind> for Option<fastn_section::KindedName> {
    fn from(value: fastn_section::Kind) -> Self {
        Some(fastn_section::KindedName {
            kind: None,
            name: value.to_identifier()?,
        })
    }
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::kinded_name);

    #[test]
    fn kinded_name() {
        // Basic cases
        t!("string", {"name": "string"});
        t!("string foo", {"name": "foo", "kind": "string"});

        // Generic types with plain identifiers
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

        // Underscores
        t!("string _private", {"name": "_private", "kind": "string"});
        t!("_", {"name": "_"});

        // Numbers in identifiers (valid after our identifier update)
        t!("int value123", {"name": "value123", "kind": "int"});
        t!("list<int> item_42", {"name": "item_42", "kind": {"name": "list", "args": ["int"]}});

        // Unicode identifiers
        t!("string नाम", {"name": "नाम", "kind": "string"});

        // Just a plain identifier (no kind)
        t!("list", {"name": "list"});

        // IMPORTANT: Generic types without a following name should fail
        // because they can't be converted to plain identifiers
        f!("list<int>");
        f!("map<string, int>");

        // IMPORTANT: Qualified types without a following name should also fail
        // because they contain dots/hashes which can't be in plain identifiers
        f!("module.Type");
        f!("package#Type");
    }
}
