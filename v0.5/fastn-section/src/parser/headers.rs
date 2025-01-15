pub fn headers(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Vec<fastn_section::Header> {
    let mut headers = vec![];
    let start_index = scanner.index();

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
        });

        let new_line = scanner.token("\n");
        if new_line.is_none() {
            // TODO: throw error
            scanner.reset(start_index);
            break;
        }
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

impl fastn_section::JDebug for fastn_section::Header {
    fn debug(&self) -> serde_json::Value {
        let mut o = serde_json::Map::new();
        if let Some(kind) = &self.kind {
            o.insert("kind".into(), kind.debug());
        }
        o.insert("name".into(), self.name.debug());
        if !self.value.0.is_empty() {
            o.insert("value".into(), self.value.debug());
        }
        serde_json::Value::Object(o)
    }
}

mod test {
    fastn_section::tt!(super::headers);

    #[test]
    fn tes() {
        t!("greeting: hello\n", [{"name": "greeting", "value": ["hello"]}]);
        t!(
            "greeting: hello\nwishes: Happy New Year\n",
            [
                {
                    "name": "greeting",
                    "value": ["hello"]
                },
                {
                    "name": "wishes",
                    "value": ["Happy New Year"]
                }
            ]
        );
        t!(
            "greeting: hello\nwishes: Happy New Year\n\nI am not header",
            [
                {
                    "name": "greeting",
                    "value": ["hello"]
                },
                {
                    "name": "wishes",
                    "value": ["Happy New Year"]
                }
            ],
            "\nI am not header"
        );
    }
}
