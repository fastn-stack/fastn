pub fn ses(scanner: &mut fastn_p1::parser::Scanner) -> Option<Vec<fastn_p1::SES>> {
    let mut ses = Vec::new();
    while let Some(text) = scanner.read_till_char_or_end_of_line('{') {
        ses.push(fastn_p1::SES::String(text));
        if !scanner.take('{') {
            // we have reached the end of the scanner
            break;
        }
    }
    Some(ses)
}

#[cfg(test)]
mod test {
    fastn_p1::tt!(super::ses);

    #[test]
    fn ses() {
        t!("hello", ["hello"], "");
        // t!("hello ${world}", [{ "text": "hello $" }, /* expression containing "world" */], "");
    }
}
