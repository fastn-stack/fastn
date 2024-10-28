/// example: `list<string> foo` | `foo bar` | `bar`
pub fn kinded_name(scanner: &mut fastn_p1::parser::Scanner) -> Option<fastn_p1::KindedName> {
    let kind = fastn_p1::parser::kind(scanner);
    let name = match fastn_p1::parser::identifier(scanner) {
        Some(v) => v,
        None => {
            return match kind {
                Some(kind) => Some(fastn_p1::KindedName {
                    kind: None,
                    name: kind.to_identifier()?,
                }),
                None => None,
            };
        }
    };

    Some(fastn_p1::KindedName { kind, name })
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::kinded_name);

    #[test]
    fn kind() {
        t!("string", {"name": {"module": {"package": "string"}}}, "");
    }
}
