pub fn tes(
    scanner: &mut fastn_section::Scanner<fastn_section::Document>,
) -> Option<Vec<fastn_section::Tes>> {
    let mut ses = Vec::new();
    while let Some(text) = scanner.take_till_char_or_end_of_line('{') {
        ses.push(fastn_section::Tes::Text(text));
        if !scanner.take('{') {
            // we have reached the end of the scanner
            break;
        }
    }
    Some(ses)
}

#[cfg(test)]
mod test {
    fastn_section::tt!(super::tes);

    #[test]
    fn tes() {
        t!("hello", ["hello"]);
        t!("hèllo", ["hèllo"]);
        // t!("hello ${world}", [{ "text": "hello $" }, /* expression containing "world" */], "");
    }
}
