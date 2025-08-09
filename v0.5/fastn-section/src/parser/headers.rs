pub fn headers(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Vec<fastn_section::Header> {
    let mut headers = vec![];
    let mut found_new_line_at_header_end = Some(scanner.index());

    loop {
        let index = scanner.index();
        scanner.skip_spaces();
        let kinded_name = match fastn_section::parser::kinded_name(scanner) {
            Some(kn) => kn,
            None => {
                scanner.reset(index);
                break;
            }
        };
        let colon = scanner.token(":");
        if colon.is_none() {
            scanner.reset(index);
            break;
        }

        if found_new_line_at_header_end.is_none() {
            scanner.reset(index);
            break;
        }

        scanner.skip_spaces();
        let value = fastn_section::parser::header_value(scanner).unwrap_or_default();

        // TODO: all the rest
        headers.push(fastn_section::Header {
            name: kinded_name.name,
            kind: kinded_name.kind,
            doc: None,
            visibility: None,
            condition: None,
            value,
            is_commented: false,
        });

        found_new_line_at_header_end = Some(scanner.index());
        let new_line = scanner.token("\n");
        if new_line.is_none() {
            found_new_line_at_header_end = None;
        }
    }

    // Reset the scanner before the new line
    if let Some(index) = found_new_line_at_header_end {
        scanner.reset(index);
    }

    headers
}

mod test {
    fastn_section::tt!(super::headers);

    #[test]
    fn headers() {
        t!("greeting: hello", [{"name": "greeting", "value": ["hello"]}]);
        t!("greeting: hello\n", [{"name": "greeting", "value": ["hello"]}], "\n");
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
            ],
            "\n"
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
            "\n\nI am not header"
        );
    }
}
