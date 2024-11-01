pub fn kinded_name(
    scanner: &mut fastn_parser::section::Scanner,
) -> Option<fastn_parser::KindedName> {
    let kind = fastn_parser::section::kind(scanner);
    scanner.skip_spaces();

    let name = match fastn_parser::section::identifier(scanner) {
        Some(v) => v,
        None => {
            return kind.and_then(Into::into);
        }
    };

    Some(fastn_parser::KindedName { kind, name })
}

#[cfg(test)]
mod test {
    fastn_parser::tt!(super::kinded_name);

    #[test]
    fn kinded_name() {
        t!("string", {"name": "string"});
        t!("string foo", {"name": "foo", "kind": "string"});
    }
}
