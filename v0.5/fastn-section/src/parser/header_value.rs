pub fn header_value(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<fastn_section::HeaderValue> {
    let mut ses = Vec::new();
    while let Some(text) = scanner.take_till_char_or_end_of_line('{') {
        ses.push(fastn_section::Tes::Text(text));
        if !scanner.take('{') {
            // we have reached the end of the scanner
            break;
        }
    }
    Some(fastn_section::HeaderValue(ses))
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::header_value);

    #[test]
    fn tes() {
        t!("hello", ["hello"]);
        t!("hèllo", ["hèllo"]);
        // t!("hello ${world}", [{ "text": "hello $" }, /* expression containing "world" */], "");
    }
}
