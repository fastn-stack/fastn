pub fn headers(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Vec<fastn_section::Header> {
    let mut headers = vec![];

    loop {
        let index = scanner.index();
        scanner.skip_spaces();
        let (kind, name) = match header_kinded_name(scanner) {
            (kind, Some(name)) => (kind, name),
            (_, None) => {
                scanner.reset(index);
                break;
            }
        };
        let colon = scanner.token(":");
        if colon.is_none() {
            scanner.reset(index);
            break;
        }

        scanner.skip_spaces();
        let value = fastn_section::parser::header_value(scanner).unwrap_or_default();

        // TODO: all the rest
        headers.push(fastn_section::Header {
            name,
            kind,
            doc: None,
            visibility: None,
            condition: None,
            value,
            is_commented: false,
        })
    }

    headers
}

pub fn header_kinded_name(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> (
    Option<fastn_section::Kind>,
    Option<fastn_section::Identifier>,
) {
    let kind = fastn_section::parser::kind(scanner);
    scanner.skip_spaces();

    let name = match fastn_section::parser::identifier(scanner) {
        Some(v) => v,
        None => {
            return (None, kind.and_then(Into::into));
        }
    };

    (kind, Some(name))
}

impl From<fastn_section::Kind> for Option<fastn_section::Identifier> {
    fn from(value: fastn_section::Kind) -> Self {
        value.to_identifier()
    }
}
