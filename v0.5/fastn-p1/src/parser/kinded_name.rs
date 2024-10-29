pub fn kinded_name<'input>(
    scanner: &'input mut fastn_p1::parser::Scanner<'input>,
) -> Option<fastn_p1::KindedName<'input>> {
    let kind = fastn_p1::parser::kind(scanner);
    scanner.skip_spaces();

    let name = match fastn_p1::parser::identifier(scanner) {
        Some(v) => v,
        None => {
            return kind.and_then(Into::into);
        }
    };

    Some(fastn_p1::KindedName { kind, name })
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::kinded_name);

    #[test]
    fn kinded_name() {
        t!("string", {"name": "string"});
        t!("string foo", {"name": "foo", "kind": "string"});
    }
}
