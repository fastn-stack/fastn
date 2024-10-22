// returns true if there is no more output left to process
pub fn module_doc(scanner: &mut fastn_p1::parser_v3::scanner::Scanner) -> bool {
    // first gobble up all the comments and empty lines
    scanner.gobble();

    // keep index in case this doc comment actually belongs to a section
    let index = scanner.index();

    if let Some(span) = scanner.take_consecutive(fastn_p1::Token::DocCommentLine) {
        if scanner.next_is(fastn_p1::Token::DashDash) {
            // this is a section doc comment, reset the scanner
            scanner.reset(index);
            return false; // since we found stuff, we are not done
        }

        scanner.output.module_doc = Some(span);
    }

    scanner.is_done()
}
