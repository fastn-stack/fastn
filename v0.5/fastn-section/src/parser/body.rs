pub fn body(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::HeaderValue> {
    let mut ses = Vec::new();
    let start = scanner.index();
    let mut reset_index = scanner.index();
    while scanner.one_of(&["-- ", "/--"]).is_none() {
        scanner.take_till_char_or_end_of_line('{');

        if scanner.peek() == Some('{') {
            todo!();
        }

        if !scanner.take('\n') {
            // we have reached the end of the scanner
            ses.push(fastn_section::Tes::Text(scanner.span(start)));
            return Some(fastn_section::HeaderValue(ses));
        }

        reset_index = scanner.index();
    }

    scanner.reset(reset_index);
    ses.push(fastn_section::Tes::Text(scanner.span(start)));
    Some(fastn_section::HeaderValue(ses))
}

mod test {
    fastn_section::tt!(super::body);

    #[test]
    fn body() {
        t!("hello world", ["hello world"]);
        t!("hello \n world", ["hello \n world"]);
        t!("hello \n world\n-- foo:", ["hello \n world\n"], "-- foo:");
        t!(
            "hello \n world\n\n-- foo:",
            ["hello \n world\n\n"],
            "-- foo:"
        );
        t!(
            "hello \n world\n\n/-- foo:",
            ["hello \n world\n\n"],
            "/-- foo:"
        );
    }
}
