pub fn kinded_name(
    scanner: &mut fastn_lang::Scanner<fastn_lang::token::Document>,
) -> Option<fastn_lang::token::KindedName> {
    let kind = fastn_lang::token::kind(scanner);
    scanner.skip_spaces();

    let name = match fastn_lang::token::identifier(scanner) {
        Some(v) => v,
        None => {
            return kind.and_then(Into::into);
        }
    };

    Some(fastn_lang::token::KindedName { kind, name })
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
