pub fn ses(scanner: &mut fastn_parser::section::Scanner) -> Option<Vec<fastn_parser::SES>> {
    let mut ses = Vec::new();
    while let Some(text) = scanner.take_till_char_or_end_of_line('{') {
        ses.push(fastn_parser::SES::String(text));
        if !scanner.take('{') {
            // we have reached the end of the scanner
            break;
        }
    }
    Some(ses)
}

#[cfg(test)]
mod test {
    fastn_parser::tt!(super::ses);

    #[test]
    fn ses() {
        t!("hello", ["hello"]);
        t!("hèllo", ["hèllo"]);
        // t!("hello ${world}", [{ "text": "hello $" }, /* expression containing "world" */], "");
    }
}
