pub fn kinded_name(
    scanner: &mut fastn_section::Scanner<fastn_section::token::Document>,
) -> Option<fastn_section::token::KindedName> {
    let kind = fastn_section::token::kind(scanner);
    scanner.skip_spaces();

    let name = match fastn_section::token::identifier(scanner) {
        Some(v) => v,
        None => {
            return kind.and_then(Into::into);
        }
    };

    Some(fastn_section::token::KindedName { kind, name })
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
