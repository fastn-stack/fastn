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
        t!("string", {"name": "string"});
        t!("string foo", {"name": "foo", "kind": "string"});
    }
}
