pub fn kinded_name(
    scanner: &mut fastn_lang::Scanner<fastn_lang::section::Document>,
) -> Option<fastn_lang::section::KindedName> {
    let kind = fastn_lang::section::kind(scanner);
    scanner.skip_spaces();

    let name = match fastn_lang::section::identifier(scanner) {
        Some(v) => v,
        None => {
            return kind.and_then(Into::into);
        }
    };

    Some(fastn_lang::section::KindedName { kind, name })
}

#[cfg(test)]
mod test {
    fastn_lang::tt!(super::kinded_name);

    #[test]
    fn kinded_name() {
        t!("string", {"name": "string"});
        t!("string foo", {"name": "foo", "kind": "string"});
    }
}
