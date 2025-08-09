pub fn kinded_name(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::KindedName> {
    let kind = fastn_section::parser::kind(scanner);
    scanner.skip_spaces();

    let name = match fastn_section::parser::identifier_reference(scanner) {
        Some(v) => v,
        None => {
            return kind.and_then(Into::into);
        }
    };

    Some(fastn_section::KindedName { kind, name })
}

impl From<fastn_section::Kind> for Option<fastn_section::KindedName> {
    fn from(value: fastn_section::Kind) -> Self {
        Some(fastn_section::KindedName {
            kind: None,
            name: value.to_identifier_reference()?,
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
    }
}
